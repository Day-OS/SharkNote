use rusqlite::{Connection, Result, Params, params};
use std::fs::File;
use std::io::prelude::*;
use std::error::Error;
//https://github.com/ProseMirror/prosemirror
//https://docs.rs/rusqlite/latest/rusqlite/index.html
//https://docs.rs/libaes/latest/libaes/struct.Cipher.html#


static DEFAULT_DIR: &'static str = "/home/ubuntu/.daytheipc-com/db/";




pub fn get_db_file_dir(db_file_name: &str) -> String{
    let mut dir: String = DEFAULT_DIR.to_string();
    dir.push_str(db_file_name.clone());
    dir.push_str(".db");
    dir
}


//THIS GETS A DATABASE AND CHECKS IF IT EXISTS
pub fn get_database (db_name: &str, rules: &str) -> Connection{
    let dir: String = get_db_file_dir(db_name);

    let file = File::open(dir.clone());
    let connection = Connection::open(dir).expect("Database to be get or created");
    match file {
        Ok(_) => {println!("{} already exists! Skipping this step.",db_name.clone());},
        Err(_) => {println!("{} does not exists! Creating database file.", db_name.clone());
            
            match connection.execute(rules, params![] // empty list of parameters.
            ) {Ok(_) => {},Err(e) => {println!("Error happened when trying to create {} tables: \n{}", db_name.clone(),e)},
            }
        },
    }
    connection
}