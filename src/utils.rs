use rocket::tokio::fs::read_dir;
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

#[derive(Serialize)]
pub struct Directory{
    pub(crate) name: String,
    pub(crate) directories: Vec<Directory>,
    pub(crate) files: Vec<FileInfo>
}

#[derive(Serialize)]
pub struct FileInfo{
    pub(crate) name: String,
    pub(crate) path: String,
}

pub fn get_directory_file_names(root: String, name: Option<String>)-> Directory{
    let mut dir: Directory = Directory { 
        name: name.unwrap_or("root".into()),
        directories: vec![],
        files: vec![] 
    };
    for path in fs::read_dir(root).unwrap().filter_map(|p| p.ok()){
        let metadata = path.metadata().unwrap();
        if metadata.is_dir() {
            dir.directories.push(get_directory_file_names(path.path().display().to_string(), Some(path.file_name().into_string().unwrap())));
        }
        else if metadata.is_file() {
            let file = FileInfo{
                name: path.file_name().into_string().unwrap(),
                path: path.path().display().to_string()
            };
            dir.files.push(file);
        }
    };
    dir
}



