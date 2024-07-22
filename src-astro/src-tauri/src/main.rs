// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::{Read, Write};
use tauri::{Emitter as _, Listener as _, Window};

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
    eprintln!("Listening on {local_addr}");

    std::thread::spawn(move || {
        let (socket, addr) = listener.accept().unwrap();
        eprintln!("Accepted connection from {addr}");
        window.emit("RECEIVE", addr.to_string()).unwrap();
        block_on_tcp("serve", window.clone(), socket).unwrap();
    });

    Ok(local_addr)
}

fn block_on_tcp(
    mode: &'static str,
    window: Window,
    mut socket: std::net::TcpStream,
) -> Result<(), String> {
    let (tx, rx) = std::sync::mpsc::channel::<String>();

    window.listen("SEND", move |e| {
        eprintln!("[{mode}:SEND] {e:?}");
        tx.send(e.payload().to_owned()).unwrap();
    });
    socket.set_read_timeout(Some(std::time::Duration::from_millis(100))).unwrap();
    let mut buf = vec![0; u16::MAX as usize];
    loop {
        if let Ok(s) = rx.try_recv() {
            socket.write_all(s.as_bytes()).unwrap();
            eprintln!("[{mode}] Sent {s:?}");
        } else if let Ok(s) = read_msg(&mut socket, &mut buf, mode) {
            window.emit("RECEIVE", &s).unwrap();
            eprintln!("[{mode}:RECEIVE] {s:?}");
        }
        std::thread::sleep(std::time::Duration::from_millis(1000));
        println!("polling...");
    }
}

fn err2str(err: impl std::error::Error) -> String {
    err.to_string()
}

fn read_msg<'b>(
    socket: &mut std::net::TcpStream,
    buf: &'b mut [u8],
    mode: &'static str,
) -> std::io::Result<std::borrow::Cow<'b, str>> {
    let mut len = 0;
    loop {
        match socket.read(&mut buf[len..]) {
            Ok(0) => break,
            Ok(n) => {
                eprintln!("[{mode}:RECEIVE] Read {n} bytes");
                len += n
            }
            Err(e) => {
                eprintln!("[{mode}:RECEIVE] {e}");
                return Err(e);
            }
        }
    }
    if len == 0 {
        eprintln!("[{mode}:RECEIVE] Tried reading from socket");
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Read 0 bytes",
        ));
    }
    Ok(String::from_utf8_lossy(&buf[..len]))
}
