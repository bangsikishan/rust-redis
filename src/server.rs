use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::fs;

use crate::redis::Redis;

pub struct Server {
    map: Arc<Mutex<Redis>>
}

impl Server {
    pub fn new() -> Self {
        match fs::read_to_string("database.db") {
            Ok(data) => {
                let mut redis = Redis::new();

                for line in data.lines() {
                    if let Some((key, value)) = line.split_once(",") {
                        redis.set(key, value);
                    }
                }

                Self {
                    map: Arc::new(Mutex::new(redis))
                }
            },
            Err(_) => Self { map: Arc::new(Mutex::new(Redis::new())) }
        }
    }

    pub fn listen(&mut self) {
        let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
        println!("[+] Server running. Listening on 127.0.0.1:6379...");

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    println!("[+] New connection established!");

                    let arc = Arc::clone(&self.map);

                    spawn(move || {
                        let mut reader = BufReader::new(stream.try_clone().unwrap());

                        loop {
                            let mut line = String::new();

                            match reader.read_line(&mut line) {
                                Ok(size) if size > 0 => {
                                    let msg = line.trim();
                                    
                                    if msg.is_empty() {
                                        continue;
                                    }

                                    println!("[+] Received: {}", msg);

                                    let parts: Vec<&str> = msg.split_whitespace().collect();

                                    match parts.get(0).map(|s| s.to_uppercase()) {
                                        Some(cmd) => match cmd.as_str() {
                                            "GET" => {
                                                if let Some(key) = parts.get(1) {
                                                    let response = {
                                                        let store = arc.lock().unwrap();

                                                        store
                                                            .get(key)
                                                            .map(|v| v.to_string())
                                                            .unwrap_or_else(|| "No record found!".to_string())
                                                    };
                                                    
                                                    let formatted = format!("{}\n", response);

                                                    stream.write_all(formatted.as_bytes()).unwrap();
                                                } else {
                                                    stream.write_all("Error: Missing key\n".as_bytes()).unwrap();
                                                }
                                            },
                                            "SET" => {
                                                if let (Some(key), Some(value)) = (parts.get(1), parts.get(2)) {
                                                    {
                                                        let mut store = arc.lock().unwrap();
                                                        store.set(key, value);
                                                        store.save_to_disk();
                                                    }

                                                    stream.write_all("Value added successfully!\n".as_bytes()).unwrap();
                                                } else {
                                                    stream.write_all("Error: Missing key or value\n".as_bytes()).unwrap();
                                                }
                                            },
                                            "DELETE" => {
                                                if let Some(key) = parts.get(1) {
                                                    {
                                                        let mut store = arc.lock().unwrap();
                                                        store.remove(key);
                                                        store.save_to_disk();
                                                    }

                                                    stream.write_all("Value deleted successfully!\n".as_bytes()).unwrap();
                                                } else {
                                                    stream.write_all("Error: Missing key\n".as_bytes()).unwrap();
                                                }
                                            },
                                            _ => stream.write_all("Unknown command\n".as_bytes()).unwrap()
                                        },
                                        None => {}
                                    }
                                },
                                Ok(_) => {
                                    println!("[+] Client disconnected!");
                                    break;
                                },
                                Err(e) => {
                                    println!("[-] Failed to read from connection: {}", e);
                                    break;
                                }
                            }
                        }
                    });
                },
                Err(e) => {
                    println!("[-] Connection failed: {}", e);
                }
            }
        }
    }
}