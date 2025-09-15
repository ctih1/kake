#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{pin::Pin, process::Command, time::Duration};

use log::{error, info, trace, warn};
use windows::Win32::UI::{Input::KeyboardAndMouse::{SendInput, INPUT, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEINPUT, VK_CAPITAL, VK_F4, VK_L, VK_LWIN, VK_MENU}, WindowsAndMessaging::SetCursorPos};
use async_trait::async_trait;
use ezsockets::{client::ClientCloseMode, ClientConfig, CloseFrame};

mod actions;

struct Client {
    handle: ezsockets::Client<Self>
}

#[async_trait]
impl ezsockets::ClientExt for Client {
    type Call = ();

    async fn on_text(&mut self, text: ezsockets::Utf8Bytes) -> Result<(), ezsockets::Error> {
        info!("received message: {text}");

        if text == "ping" {
            let username = std::env::var("USERNAME").unwrap();
            let _ = self.handle.text(format!("username={username}"));
        }
        // messages should be in the following format: key=val;key2=val2;
        let parts: Vec<&str> =  text.split(";").collect();  
        let mut action: String = String::new();
        let mut param: String = String::new();
        let mut val: String = String::new();

        for arg in parts {
            let parts: Vec<&str> = arg.split("=").collect();
            
            if parts.len() != 2 {
                let res = self.handle.text(format!("Invalid body; expected 2 parts for arg {}", arg).to_string());
                if let Err(_) = res {
                    error!("Failed to send invalid request body");
                }
                continue;
            }
            
            let [key, value] = parts.as_slice().try_into().unwrap();

            match key {
                "action" => { action = value.to_string(); }
                "param" => { param = value.to_string(); }
                "value" => { val = value.to_string(); }
                _ => {
                    warn!("{}",format!("Invalid key {}!", key))
                }
            }
        }

        
        if action == "do" {
            match param.as_str() {
                "caps" => { 
                    info!("Toggling caps");
                    actions::toggle_caps();
                    let _ = self.handle.text("ok".to_string());
                },
                "lock" => {
                    actions::win_lock();
                    let _ = self.handle.text("ok");
                }
                "close" => {
                    info!("Closing app");
                    actions::alt_f4();
                    let _ = self.handle.text("ok".to_string());
                }
                "link" => {
                    info!("Opening link {}", val);
                    Command::new("cmd").args(["/C", format!("start {}", val).as_str()]).spawn().unwrap();
                    let _ = self.handle.text("ok".to_string());
                }
                "mouse" => {
                    let parts: Vec<&str> = val.split(",").collect();
                    unsafe {
                        if let Err(e) =  SetCursorPos(parts[0].parse().unwrap(), parts.last().unwrap().parse().unwrap()) {
                            warn!("Failed to update pos {}",e )
                        }
                    }
                    let _ = self.handle.text("ok".to_string());
                }
                "ldown" => {
                    let parts: Vec<&str> = val.split(",").collect();
                    unsafe {
                        let mut click = INPUT {
                            r#type: INPUT_MOUSE,
                            Anonymous: std::mem::zeroed(),
                        };
                        click.Anonymous.mi = MOUSEINPUT {
                            dx: parts[0].parse().unwrap(),
                            dy: parts[0].parse().unwrap(),
                            dwExtraInfo: 0,
                            dwFlags: MOUSEEVENTF_LEFTDOWN,
                            time: 0,
                            mouseData: 0
                        };
                        let inputs = vec![click];
                        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
                    }
                }
                "lup" => {
                    let parts: Vec<&str> = val.split(",").collect();
                    unsafe {
                        let mut click = INPUT {
                            r#type: INPUT_MOUSE,
                            Anonymous: std::mem::zeroed(),
                        };
                        click.Anonymous.mi = MOUSEINPUT {
                            dx: parts[0].parse().unwrap(),
                            dy: parts[0].parse().unwrap(),
                            dwExtraInfo: 0,
                            dwFlags: MOUSEEVENTF_LEFTUP,
                            time: 0,
                            mouseData: 0
                        };
                        
                        let inputs = vec![click];
                        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
                    }
                }
                _ => { warn!("Invalid arg {}", param); }
            }
        }
        
        else if action == "get" {
            let _ = self.handle.text("Getting data".to_string());
        } else {
            warn!("Invalid action {}", action);
        }
        Ok(())
    }

    async fn on_binary(&mut self, bytes: ezsockets::Bytes) -> Result<(), ezsockets::Error> {
        trace!("received bytes: {bytes:?}");
        Ok(())
    }

    async fn on_call(&mut self, call: Self::Call) -> Result<(), ezsockets::Error> {
        let () = call;
        Ok(())
    }

    async fn on_connect(&mut self) -> Result<(), ezsockets::Error> {
        info!("Connected to server");
        info!("Sending registration");
        let username = std::env::var("USERNAME").unwrap();
        let _ = self.handle.text(format!("name={username}"));
        Ok(())
    }
    
    fn on_close<'life0, 'async_trait>(
        &'life0 mut self,
        _frame: Option<CloseFrame>,
    ) -> Pin<Box<dyn Future<Output = Result<ClientCloseMode, ezsockets::Error>> + Send + 'async_trait>>
    where
        Self: 'async_trait,
        'life0: 'async_trait,
    {
        warn!("Lost connection to server");

        Box::pin(async { Ok(ClientCloseMode::Reconnect) })
    }

}


#[tokio::main]
async fn main() {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Debug).init();

    info!("Setting client config");
    let cfg = ClientConfig::new("ws://koti.mp4.fi:8040/ws");
    let config = cfg.socket_config(ezsockets::SocketConfig { heartbeat: Duration::from_secs(3), timeout: Duration::from_secs(8), ..Default::default() })
        .reconnect_interval(Duration::from_secs(3))
        .max_reconnect_attempts(9999999);

    info!("Attempting to connect");
    let (_handle, future) = ezsockets::connect(|handle| Client { handle }, config).await;

    future.await.unwrap();
}
