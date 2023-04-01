use crate::db_utils;
use db_utils::DBErrors;
use rocket::State;
use rusqlite::{Connection, params};



#[derive(Debug)]
pub struct PageOwner {
    pub page_owner_id: String,
    pub password: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub profile_picture: Option<Vec<u8>>,
    pub is_program_admin: u8,
}

//CREATE PASSWORD ENCRYPTION LATER!
//pub fn encrypt

pub fn get_database() -> Connection{
    db_utils::get_database("page_owners", "
    CREATE TABLE page_owner (
        page_owner_id TEXT PRIMARY KEY,
        password TEXT NOT NULL,
        display_name TEXT UNIQUE,
        description TEXT,
        profile_picture BLOB,
        is_program_admin INTEGER NOT NULL
    )")
}
/*
fn encrypt_password(password: &str)-> String {
    let secret_key = crate::config_file::get_config_file().secret_key;
    Cipher::new_128(secret_key.as_bytes());
};
*/

//let errormsg: String = format!("Could not retrieve PageOwner with id {}.", id);
pub fn get_user_from_id(c : &Connection, id : &str) -> Result<PageOwner,  DBErrors>{
    let mut statement = match c.prepare("SELECT * FROM page_owner where page_owner_id = ?1"){
        Ok(statement) => statement,
        Err(e) => return Err(DBErrors::SqliteError(e))
    };

    let users = match statement.query_map([id], |row| 
    {
        Ok(PageOwner { 
        page_owner_id: row.get(0)?, 
        password: row.get(1)?, 
        display_name: row.get(2)?, 
        description: row.get(3)?, 
        profile_picture: row.get(4)?,
        is_program_admin: row.get(5)?})
    }){
        Ok(users )=> users,
        Err(e)=> return Err(DBErrors::SqliteError(e))
    };
    
    for user in users{ 
        match user {
            Ok(page_owner)=>{
                return Ok(page_owner);
            }
            Err(e)=>{
                return Err(DBErrors::SqliteError(e))
            }
        }
    
    }
    return Err(DBErrors::GenericNotFound)

} 


pub fn create_user(c : &Connection, owner : PageOwner){
    let command = c.execute("INSERT INTO page_owner (page_owner_id, password, is_program_admin) VALUES (?1, ?2, ?3)", (&owner.page_owner_id, &owner.password, &owner.is_program_admin));
    match command {
        Ok(_)=>{}
        Err(e)=>{println!("Could not create PageOwner with that arguments! | {}", e)}
    }
}

pub fn change_password(c : &Connection, id : &str, password : &str){
    let command = c.execute("UPDATE page_owner SET password = ?1 WHERE page_owner_id = ?2", params![password, id]);
    match command {
        Ok(_)=>{}
        Err(e)=>{
            println!("Could not change {}'s password! | {}", id, e)}
    }
}

pub fn delete_user(c : &Connection, id : &str) -> Result<(), db_utils::DBErrors>{
    let command = c.execute(
        "DELETE FROM page_owner WHERE page_owner_id = ?1", params![id]);
    match command {
        Ok(_)=> Ok(()),
        Err(e)=> Err(db_utils::DBErrors::CantDeleteUser(id.to_owned(), e))
    }
}

pub fn check_login_credentials( username : &str, password : &str , db: &State<crate::DbConn>) -> Result<bool, db_utils::DBErrors>{ 
    match db.lock() {
        Ok(database) => {
            match get_user_from_id(&database, username) {
                Ok(page_owner) => {
                    let result:bool = (page_owner.page_owner_id == username) && (page_owner.password == password);
                    return Ok(result);
                }
                Err(e)=>{return Err(e);}
            }
        },
        Err(e) => {
            return Err(db_utils::DBErrors::ProgramError(e.to_string()));
        }
    }
}