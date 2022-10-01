//! Functions for creating directories and storing files

use std::{
    fs::{create_dir_all, File},
    io::{BufReader, Read, Write},
};

/// Wrapper for configuration file.
pub struct Config {
    content: String,
    file: File,
}

impl Config {
    pub fn add_line() {
        
    }
    pub fn save_config() {
        
    }
}

fn cache_dir() -> Option<String> {
    let hd = dirs::home_dir().map(|p| Some(p.to_str()?.to_string()))??;
    Some(format!("{}/.cache/sc/", hd))
}

fn cfg_dir() -> Option<String> {
    let hd = dirs::home_dir().map(|p| Some(p.to_str()?.to_string()))??;
    Some(format!("{}/.config/sc/", hd))
}

fn cfg_path() -> Option<String> {
    Some(format!("{}config", cfg_dir()?))
}

/// Reads the contents of the config file and returns the file handle.
/// If no config was found, creates a new config file.
pub fn init_config() -> Config {
    let cfg_file = match File::options()
        .read(true)
        .write(true)
        .open(cfg_path().unwrap())
    {
        Ok(f) => f,
        Err(_) => create_config(),
    };
    let mut buf = String::new();
    BufReader::new(&cfg_file).read_to_string(&mut buf).unwrap();
    Config{ content: buf, file: cfg_file }
}
/// Creates a config file. Only call this when you know that no
/// config file exists already.
fn create_config() -> File {
    create_dir_all(cfg_dir().unwrap()).expect("create .config directory for sc");
    File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(cfg_path().unwrap())
        .expect("Create file in cfg directory")
}
