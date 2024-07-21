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
    let mut socket = std::net::TcpStream::connect(addr).map_err(err2str)?;
    
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    
    let send_tx = tx.clone();
    let send = window.listen("SEND", move |e| {
        send_tx.send(e.payload().to_owned()).unwrap();
    });
    socket.set_nonblocking(true).map_err(err2str)?;
    std::thread::spawn(move || {
        let mut buf = Vec::with_capacity(u16::MAX as usize);
        let err = loop {
            if let Ok(s) = rx.try_recv() {
                if let Err(e) = socket.write_all(s.as_bytes()) {
                    break e.to_string();
                }
            } else if socket.read_to_end(&mut buf).is_ok() {
                if let Err(e) = window.emit("RECEIVE", String::from_utf8_lossy(&buf)) {
                    break e.to_string();
                }
            }
        };
        
        println!("{err}");
        window.unlisten(send);
    });
    
    Ok(())
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn serve(window: Window) -> Result<String, String> {
    let local_ip = local_ip_address::local_ip().unwrap();
    let listener = std::net::TcpListener::bind("0.0.0.0:0").map_err(err2str)?;

    let local_addr = {
        let mut a = listener.local_addr().unwrap();
        a.set_ip(local_ip);
        a.to_string() 
    };
    println!("Listening on {local_addr}");
    
    std::thread::spawn(move || {
        let (mut socket, addr) = listener.accept().unwrap();
        println!("Accepted connection from {addr}");
        window.emit("RECEIVE", addr.to_string()).unwrap();
        
        let (tx, rx) = std::sync::mpsc::channel::<String>();
        let send_tx = tx.clone();
        let send = window.listen("SEND", move |e| {
            send_tx.send(e.payload().to_owned()).unwrap();
        });
        socket.set_nonblocking(true).unwrap();

        let mut buf = Vec::with_capacity(u16::MAX as usize);
        let err = loop {
            if let Ok(s) = rx.try_recv() {
                if let Err(e) = socket.write_all(s.as_bytes()) {
                    break e.to_string();
                }
            } else if socket.read_to_end(&mut buf).is_ok() {
                if let Err(e) = window.emit("RECEIVE", String::from_utf8_lossy(&buf)) {
                    break e.to_string();
                }
            }
        };
        
        println!("{err}");
        window.unlisten(send);
    });

    Ok(local_addr)
}

fn err2str(err: impl std::error::Error) -> String {
    err.to_string()
}