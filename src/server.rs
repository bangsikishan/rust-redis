use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use crate::redis::Redis;

pub struct Server {
    map: Redis
}

impl Server {
    pub fn new() -> Self {
        Self {
            map: Redis::new()
        }
    }

    pub fn listen(&mut self) {
        let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
        println!("[+] Server running. Listening on 127.0.0.1:6379...");

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    println!("[+] New connection established!");

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
                                                let response = self.map
                                                .get(key)
                                                .map(|v| v.as_str())
                                                .unwrap_or("No record found!");
                                                
                                                let formatted = format!("{}\n", response);

                                                stream.write_all(formatted.as_bytes()).unwrap();
                                            } else {
                                                stream.write_all("Error: Missing key\n".as_bytes()).unwrap();
                                            }
                                        },
                                        "SET" => {
                                            if let (Some(key), Some(value)) = (parts.get(1), parts.get(2)) {
                                                self.map.set(
                                                    key,
                                                    value
                                                );

                                                stream.write_all("Value added successfully!".as_bytes()).unwrap();
                                            } else {
                                                stream.write_all("Error: Missing key or value\n".as_bytes()).unwrap();
                                            }
                                        },
                                        "DELETE" => {
                                            if let Some(key) = parts.get(1) {
                                                self.map.remove(key);

                                                stream.write_all("Value deleted successfully!".as_bytes()).unwrap();
                                            } else {
                                                stream.write_all("Error: Missing key\n".as_bytes()).unwrap();
                                            }
                                        },
                                        _ => stream.write_all("Unknown command".as_bytes()).unwrap()
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
                },
                Err(e) => {
                    println!("[-] Connection failed: {}", e);
                }
            }
        }
    }
}