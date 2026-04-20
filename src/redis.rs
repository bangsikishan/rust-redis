use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

pub struct Redis {
    map: HashMap<String, String>
}

impl Redis {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.map.insert(key.to_string(), value.to_string());
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.map.get(key)
    }

    pub fn remove(&mut self, key: &str) {
        self.map.remove(key);
    }

    pub fn save_to_disk(&self) {
        match File::create("database.db") {
            Ok(mut file) => {
                for (key, value) in &self.map {
                    writeln!(file, "{},{}", key, value).unwrap();
                }
            },
            Err(e) => eprintln!("Failed to save file: {}", e)
        }
    }
}