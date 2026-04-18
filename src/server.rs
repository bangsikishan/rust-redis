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
                                println!("[+] Received: {}", msg);

                                let parts: Vec<&str> = msg.split_whitespace().collect();
                                
                                match parts[0].to_uppercase().as_str() {
                                    "GET" => {
                                        let response = self.map
                                        .get(parts[1])
                                        .map(|v| v.as_str())
                                        .unwrap_or("No record found!");

                                        stream.write_all(response.as_bytes()).unwrap();
                                    },
                                    "SET" => {
                                        self.map
                                        .set(
                                            parts[1].to_lowercase().as_str(),
                                            parts[2].to_lowercase().as_str()
                                        );

                                        stream.write_all("Value added successfully!".as_bytes()).unwrap();
                                    },
                                    "UPDATE" => {
                                        self.map.update(parts[1], parts[4]);
                                        stream.write_all("Value updated successfully!".as_bytes()).unwrap();
                                    },
                                    "DELETE" => {
                                        self.map
                                        .remove(parts[1]);
                                        stream.write_all("Value deleted successfully!".as_bytes()).unwrap();
                                    },
                                    _ => println!("Unknown command")
                                }

                                stream.write_all(b"OK\n").unwrap();
                            },
                            Ok(_) => println!("[+] Client disconnected!"),
                            Err(e) => println!("[-] Failed to read from connection: {}", e)
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