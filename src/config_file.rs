use std::{path::Path, fs};
use ron;
use dirs;
static PROGRAM_NAME: &str = "daytheipc-com";
use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize)]
pub struct Config{
    pub secret_key: String,
    pub port: u16,
    
}
pub fn get_dirs_database_location() -> String{
    let mut path = dirs::home_dir().unwrap().to_str().unwrap().to_string();
    path.push_str("/");
    path.push_str(PROGRAM_NAME);
    path.push_str("/db/");
    path
}
pub fn get_config_file() -> Config{
    let mut path = dirs::home_dir().unwrap().to_str().unwrap().to_string();
    path.push_str("/");
    path.push_str(PROGRAM_NAME);
    path.push_str("/configuration.ron");
    let s = fs::read_to_string(Path::new(&path)).unwrap();
    let ron: Config = ron::from_str(s.as_str()).unwrap();
    ron
}