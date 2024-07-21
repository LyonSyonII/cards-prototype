// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{io::{Read, Write}, net::Ipv4Addr, str::FromStr};
use tauri::{Emitter as _, Event, Listener as _, Window};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![connect, serve])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn connect(window: Window, addr: &str) -> Result<(), String> {
    let socket = std::net::TcpStream::connect(addr).map_err(err2str)?;
    std::thread::spawn(|| block_on_tcp("connect", window, socket));
    Ok(())
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn serve(window: Window) -> Result<String, String> {
    let local_ip = local_ip_address::local_ip().map_err(err2str)?;
    let listener = std::net::TcpListener::bind("0.0.0.0:0").map_err(err2str)?;

    let local_addr = {
        let mut a = listener.local_addr().map_err(err2str)?;
        a.set_ip(local_ip);
        a.to_string() 
    };
    println!("Listening on {local_addr}");
    
    std::thread::spawn(move || {
        let (socket, addr) = listener.accept().unwrap();
        println!("Accepted connection from {addr}");
        window.emit("RECEIVE", addr.to_string()).handle_err(&window);
        block_on_tcp("serve", window.clone(), socket).handle_err(&window);
    });
    
    Ok(local_addr)
}

fn block_on_tcp(mode: &'static str, window: Window, mut socket: std::net::TcpStream) -> Result<(), String> {
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    let send_tx = tx.clone();
    window.listen("SEND", move |e| {
        send_tx.send(e.payload().to_owned()).unwrap();
    });
    
    let mut buf = Vec::with_capacity(u16::MAX as usize);
    loop {
        if let Ok(s) = rx.try_recv() {
            socket.write_all(s.as_bytes()).handle_err(&window);
        } else if socket.read_to_end(&mut buf).is_ok_and(|b| b > 0) {
            let r = String::from_utf8_lossy(&buf);
            window.emit("RECEIVE", &r).handle_err(&window);
            println!("[{mode}] Received {r:?}");
        }
    };
}

fn err2str(err: impl std::error::Error) -> String {
    err.to_string()
}

trait ResultExt where Self: Sized {
    fn handle_err(self, window: &Window);
}

impl<T, E: ToString> ResultExt for Result<T, E> {
    fn handle_err(self, window: &Window) {
        if let Err(e) = self {
            window.emit("RECEIVE", e.to_string()).unwrap();
        }
    }
}