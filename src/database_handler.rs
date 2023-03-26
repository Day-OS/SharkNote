use rusqlite::{Connection, Result};
use std::fs::File;
use std::io::prelude::*;
use std::error::Error;
//https://github.com/ProseMirror/prosemirror
//https://docs.rs/rusqlite/latest/rusqlite/index.html
//https://docs.rs/libaes/latest/libaes/struct.Cipher.html#


static DEFAULT_DIR: &'static str = "/home/ubuntu/.daytheipc-com/db/";

#[derive(Debug)]
struct Pages {
    page_name: String,
    page_configuration: String,
    data: Option<Vec<u8>>,
}


#[derive(Debug)]
struct Files {
    file_name: String,
    page_name: String,
    data: Option<Vec<u8>>,
}


fn get_page_name(page_name: &str) -> String{
    let mut dir: String = DEFAULT_DIR.to_string();
    dir.push_str(page_name.clone());
    dir.push_str(".db");
    dir
}

pub fn create_table_if_not_exists (page_name: &str) -> Result<()>{
    let dir: String = get_page_name(page_name);

    let file = File::open(dir.clone());
    match file {
        Ok(_) => {println!("{} already exists! Skipping this step.",page_name.clone())},
        Err(_) => {
            println!("{} does not exists! Creating database file.", page_name.clone());
            let conn = Connection::open(dir)?;

            match conn.execute(
                "CREATE TABLE pages (
                        page_name TEXT PRIMARY KEY,
                        page_configuration TEXT NOT NULL,
                        data BLOB);
                    CREATE TABLE files (
                        file_name TEXT PRIMARY KEY,
                        page_name TEXT NOT NULL,
                        data BLOB NOT NULL;
                    data BLOB)", (), // empty list of parameters.
            ) {
                Ok(_) => {},
                Err(e) => {println!("Error happened when trying to create {} tables: \n{}", page_name.clone(),e)},
            }
            match conn.close() {Ok(_) => {println!("File created!")},
                Err(_) => {println!("Something bad happened when trying to close {}.", page_name.clone())},
            };
        },
    }

    Ok(())
}

pub fn get_page(page_name: &str) -> Result<Connection>{
    let dir: String = get_page_name(page_name);
    Ok(Connection::open(dir)?)
} 
    /*

    let page = Pages {
        page_name: page_name.to_string(),
        page_configuration: "{lol:0}".to_string(),
        data: None,
    };
    conn.execute(
        "INSERT INTO person (name, data) VALUES (?1, ?2)",
        (&me.name, &me.data),
    )?;

    let mut stmt = conn.prepare("SELECT id, name, data FROM person")?;
    let person_iter = stmt.query_map([], |row| {
        Ok(Person {
            id: row.get(0)?,
            name: row.get(1)?,
            data: row.get(2)?,
        })
    })?;

    for person in person_iter {
        println!("Found person {:?}", person.unwrap());
    }
    */
