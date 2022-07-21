extern crate core;
extern crate scoped_threadpool;
mod utils;
mod media;

use crate::utils::*;
use std::{time, time::SystemTime};
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::process::Command;
use std::sync::Mutex;
use std::thread::sleep;
use colored::Colorize;
use scoped_threadpool::Pool;
use crate::media::MediaState;

const INCOMING_PREFIX:&str = "ev2pc-";
const OUTGOING_PREFIX:&str = "pc2ev-";
fn main() {
    let connections:Mutex<HashMap<SocketAddr, Ev3Connection>> = Mutex::new(HashMap::new());

    let listen = "0.0.0.0:6969";
    println!("Hello, world!");

    let mut pool = Pool::new(2);

    pool.scoped(|scope| {
        scope.execute(|| socket_thread(&connections, listen));
        scope.execute(|| keepalive_thread(&connections));
        scope.join_all();
    });
}

fn socket_thread(connections: &Mutex<HashMap<SocketAddr, Ev3Connection>>, listen:&str) {
    println!("insane_thread");

    let socket = UdpSocket::bind(listen).expect("Couldn't bind to address");
    let mut buf = [0; 1024];

    let mut media_state = MediaState {
        playing: false,
        current_song: "Never Gonna Give You Up".to_string(),
        current_time: 0,
        max_time: 0
    };

    loop {
        let (amount, src) = socket.recv_from(&mut buf).expect("Couldn't receive data");
        let message = String::from_utf8_lossy(&buf[..amount-1]); // -1 to cut off the \n

        if message.starts_with(INCOMING_PREFIX) {
            let message = message.to_string().strip_prefix(INCOMING_PREFIX).unwrap().to_string();
            let split = message.split("?").collect::<Vec<&str>>();
            let command = split[0];
            match command {
                "connect" => {
                    if split.len() > 1 {
                        let mut connections = connections.lock().unwrap();
                        println!("{} {} {}", "EV3 connected,".green(), "name:".blue(), split[1].blue().underline());
                        connections.insert(src, Ev3Connection {
                            name: split[1].to_string(),
                            connected: true,
                            last_seen: SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs()
                        });

                        drop(connections);
                        send_to_ev3(&socket, &src, "connected");
                    }
                }
                "keepalive" => {
                    let mut connections = connections.lock().unwrap();
                    if connections.contains_key(&src) {
                        let connection = connections.get_mut(&src).unwrap();
                        connection.last_seen = SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs();
                    }
                    send_to_ev3(&socket, &src, "keepalive");
                    drop(connections);
                }
                "media" => {
                    if split.len() > 1 {
                        let connections = connections.lock().unwrap();
                        match split[1] {
                            "query" => {
                                send_media_state(&socket, &src, &media_state);
                            }
                            "pp" => {
                                press_key("172");
                                media_state = media::invert_playing(&media_state);
                                send_media_state(&socket, &src, &media_state);
                            }
                            "next" => {
                                press_key("171");
                                send_media_state(&socket, &src, &media_state);
                            }
                            "prev" => {
                                press_key("173");
                                send_media_state(&socket, &src, &media_state);
                            }
                            "volup" => {
                                press_key("123");
                                send_media_state(&socket, &src, &media_state);
                            }
                            "voldown" => {
                                press_key("122");
                                send_media_state(&socket, &src, &media_state);
                            }
                            "veryfunnyandhilariousmessagethatdefinitelydoesnotshutdownthewholefuckingsystemlmao" => {
                                println!("{}", "Shutting down...".red());
                                let mut command = Command::new("sudo");
                                command.arg("shutdown").arg("now");
                                command.status().unwrap();
                            }
                            _ => {}
                        }
                        drop(connections);
                    }
                }
                &_ => {
                    println!("Unknown command: {}", command);
                }
            }
        }
    }
}

fn send_media_state(socket: &UdpSocket, src: &SocketAddr, funneh: &MediaState) {
    send_to_ev3(socket, src, format!("media?result={}|{}|{}|{}",
                                     if funneh.playing { 0 } else { 1},
                                     base64::encode(&funneh.current_song),
                                     funneh.current_time, funneh.max_time).as_str());
}

fn press_key(key:&str) {
    let mut command = Command::new("xdotool");
    command.arg("key").arg(key);
    command.status().unwrap();
}

fn keepalive_thread(results: &Mutex<HashMap<SocketAddr, Ev3Connection>>) {
    loop {
        let mut results = results.lock().unwrap();
        for (addr, connection) in results.clone().iter_mut() {
            if connection.connected {
                if SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs() - connection.last_seen > 5 {
                    println!("{} {} {}", "EV3 disconnected,".red(), "name:".blue(), connection.name.blue().underline());
                    results.remove(addr);
                }
            } else {
                results.remove(addr);
            }
        }
        drop(results);
        sleep(time::Duration::from_millis(420));
    }
}

fn send_to_ev3(socket: &UdpSocket, addr: &SocketAddr, message: &str) {
    send(socket, addr, &*(OUTGOING_PREFIX.to_owned() + message));
}

fn send(socket:&UdpSocket, address:&SocketAddr, message:&str) {
    socket.send_to((message.to_owned() + "\n").as_bytes(), address).expect("Couldn't send data");
}