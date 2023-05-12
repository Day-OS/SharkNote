use crate::utils::{self, DBErrors};
use rocket::State;
use rusqlite::{Connection, params};



#[derive(Debug)]
pub struct User {
    pub user_id: String,
    pub password: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub profile_picture: Option<Vec<u8>>,
    pub is_program_admin: u8,
}


pub static CREATION_RULES: &str = "
CREATE TABLE user (
    user_id TEXT PRIMARY KEY,
    password TEXT NOT NULL,
    display_name TEXT UNIQUE,
    description TEXT,
    profile_picture BLOB,
    is_program_admin INTEGER NOT NULL
);";


//CREATE PASSWORD ENCRYPTION LATER!
//pub fn encrypt

/*
fn encrypt_password(password: &str)-> String {
    let secret_key = crate::config_file::get_config_file().secret_key;
    Cipher::new_128(secret_key.as_bytes());
};
*/

//let errormsg: String = format!("Could not retrieve User with id {}.", id);
pub fn get_user_from_id(c : &Connection, id : &str) -> Result<User,  DBErrors>{
    let mut statement = match c.prepare("SELECT * FROM user where user_id = ?1"){
        Ok(statement) => statement,
        Err(e) => return Err(DBErrors::SqliteError(e))
    };

    let users = match statement.query_map([id], |row| 
    {
        Ok(User { 
        user_id: row.get(0)?, 
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
            Ok(user)=>{
                return Ok(user);
            }
            Err(e)=>{
                return Err(DBErrors::SqliteError(e))
            }
        }
    
    }
    return Err(DBErrors::GenericNotFound)

} 


pub fn create_user(c : &Connection, owner : User){
    match c.execute
    ("INSERT INTO user (user_id, password, is_program_admin) VALUES (?1, ?2, ?3)", 
    (&owner.user_id, &owner.password, &owner.is_program_admin)) {
        Ok(_)=>{}
        Err(e)=>{println!("Could not create User with that arguments! | {}", e)}
    }
}

pub fn change_password(c : &Connection, id : &str, password : &str){
    match c.execute("UPDATE user SET password = ?1 WHERE user_id = ?2", params![password, id]) {
        Ok(_)=>{}
        Err(e)=>{
            println!("Could not change {}'s password! | {}", id, e)}
    }
}

pub fn delete_user(c : &Connection, id : &str) -> Result<(), utils::DBErrors>{
    match c.execute("DELETE FROM user WHERE user_id = ?1", params![id]) {
        Ok(_)=> Ok(()),
        Err(e)=> Err(utils::DBErrors::CantDeleteUser(id.to_owned(), e))
    }
}

pub fn check_login_credentials( username : &str, password : &str , db: &State<crate::DbConn>) -> Result<bool, utils::DBErrors>{ 
    match db.lock() {
        Ok(database) => {
            match get_user_from_id(&database, username) {
                Ok(user) => {
                    let result:bool = (user.user_id == username) && (user.password == password);
                    return Ok(result);
                }
                Err(e)=>{return Err(e);}
            }
        },
        Err(e) => {
            return Err(utils::DBErrors::ProgramError(e.to_string()));
        }
    }
}