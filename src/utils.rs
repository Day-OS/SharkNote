use rusqlite::{Connection, params};
use std::fs::File;
use std::{path::Path, fs};
use ron;
use dirs;
static PROGRAM_NAME: &str = "daytheipc-com";
use serde::{Deserialize, Serialize};


//https://github.com/ProseMirror/prosemirror
//https://docs.rs/rusqlite/latest/rusqlite/index.html
//https://docs.rs/libaes/latest/libaes/struct.Cipher.html#
pub enum DBErrors {

    CantDeleteUser(String, rusqlite::Error),
    GenericNotFound,
    ProgramError(String),
    SqliteError(rusqlite::Error)
}

//THIS GETS A DATABASE AND CHECKS IF IT EXISTS
pub fn get_database (rules: Vec<&str>) -> Connection{
    let db_name = "database"; 
    let mut dir = get_program_files_location();
    
    dir.push_str("db/");
    dir.push_str(db_name.clone());
    dir.push_str(".db");

    let file = File::open(dir.clone());
    let connection = Connection::open(dir).expect("Database to be get or created");
    match file {
        Ok(_) => {println!("{} already exists! Skipping this step.",db_name.clone());},
        Err(_) => {println!("{} does not exists! Creating database file.", db_name.clone());
        for rule in rules {
            match connection.execute(rule, params![])
            {
                Ok(_) => {},
                Err(e) => {println!("Error happened when trying to create {} tables: \n{}", db_name.clone(),e)},
            }
        }
            
        },
    }
    connection
}



#[derive(Debug, Deserialize, Serialize)]
pub struct Config{
    pub secret_key: String,
    pub port: u16,
    
}

pub fn get_program_files_location() -> String {
    let mut path = dirs::home_dir().unwrap().to_str().unwrap().to_string();
    path.push_str("/");
    path.push_str(PROGRAM_NAME);
    path.push_str("/");
    path
}

pub fn get_config_file() -> Config{
    let mut path = get_program_files_location();
    path.push_str("configuration.ron");
    let s = fs::read_to_string(Path::new(&path)).unwrap();
    let ron: Config = ron::from_str(s.as_str()).unwrap();
    ron
}



