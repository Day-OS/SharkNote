use std::{path::Path, fs};
use ron;
use dirs;
static PROGRAM_NAME: &str = "daytheipc-com";
use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize)]
pub struct Language{
    pub could_not_delete_user_error: String,
    pub program_error: String,
    pub wrong_password: String,
    
}

pub fn get_file() -> Language{
    let mut path = dirs::home_dir().unwrap().to_str().unwrap().to_string();
    path.push_str("/");
    path.push_str(PROGRAM_NAME);
    path.push_str("/language.ron");
    let s = fs::read_to_string(Path::new(&path)).unwrap();
    let ron: Language = ron::from_str(s.as_str()).unwrap();
    ron
}