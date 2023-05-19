use crate::{utils::{self, DBErrors}, user};
use rusqlite::{Connection, params};
use serde::Serialize;
use walkdir::WalkDir;

pub(crate) static PAGE_DIR: &str = "pages/";

pub static PAGE_CREATION_RULES: &str = "
CREATE TABLE page (
    page_id TEXT PRIMARY KEY,
    page_display_name TEXT,
    b_is_archived int NOT NULL DEFAULT 0
);";
pub static PAGE_TRIGGER_RULES: &str = "
CREATE TRIGGER default_page_display_name AFTER INSERT ON page
FOR EACH ROW WHEN NEW.page_display_name IS NULL
BEGIN 
    UPDATE page SET page_display_name = NEW.page_id WHERE rowid = NEW.rowid;
END;
";
pub static PAGEUSER_CREATION_RULES: &str = "
CREATE TABLE page_user (
    page_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    PRIMARY KEY (page_id, user_id)
    FOREIGN KEY (page_id) REFERENCES page(page_id) ON DELETE CASCADE ON UPDATE CASCADE
    FOREIGN KEY (user_id) REFERENCES user(user_id) ON DELETE CASCADE ON UPDATE CASCADE
);";

#[derive(Debug, Serialize)]
pub struct Page {
    pub page_id: String,
    pub page_display_name: String,
    pub b_is_archived: bool, 
}

#[derive(Debug)]
pub struct PageUser {
    pub page_id: String,
    pub user_id: String,
}

pub fn check_and_fix_page(c : &Connection, page : String){
    let mut page_dir = utils::get_program_files_location();
    page_dir.push_str(PAGE_DIR);
    page_dir.push_str(&page);
    match std::fs::read_dir(page_dir){
        Ok(_) =>{println!("page_exists")}
        Err(_) => {
            create_page(c, page);
            println!("page_doesnt_exists")
        }
    }
}


pub fn get_owned_pages(c : &Connection, user_id : &str) -> Result<Vec<Page>,  DBErrors>{
    let mut statement = match c.prepare("SELECT page.page_id, page_display_name, b_is_archived FROM page_user INNER JOIN page ON page.page_id = page_user.page_id AND user_id = ?1"){
        Ok(statement) => statement,
        Err(e) => return Err(DBErrors::SqliteError(e))
    };

    let pages_mapped = match statement.query_map([user_id], |row| 
    {
        Ok(Page{
            page_id: row.get(0)?,
            page_display_name: row.get(1)?,
            b_is_archived: row.get(2)?
        })
    }){
        Ok(pages )=> pages,
        Err(e)=> return Err(DBErrors::SqliteError(e))
    };
    let mut pages: Vec<Page> = vec![];
    for page in pages_mapped{ 
        match page {
            Ok(page)=>{
                pages.push(page);
            }
            Err(e)=>{
                return Err(DBErrors::SqliteError(e))
            }
        }
    
    }
    return Ok(pages)

} 

pub fn get_pages_files_list(page : &String)-> Vec<String> {
    let mut files = vec![];

    let mut page_dir = utils::get_program_files_location();
    page_dir.push_str(PAGE_DIR);
    page_dir.push_str(&page);

    

    for entry in WalkDir::new(&page_dir).into_iter().filter_map(|e| e.ok()) {
        let mut path: String = entry.path().display().to_string();
        path = path.replace(&page_dir, "");
        if entry.metadata().unwrap().is_file() {
            files.push(path);
        }
    }

    files
}

pub fn create_page(c : &Connection, page : String){ match c.execute("INSERT INTO page (page_id) VALUES (?1)", params![page]){
    Err(e)=>{println!("Could not create User with that arguments! | {}", e)}
    Ok(_)=>{
        let mut page_dir = utils::get_program_files_location();
        page_dir.push_str(PAGE_DIR);
        page_dir.push_str(&page);
        println!("{}", page_dir);
        std::fs::create_dir_all(page_dir);
    }
}}

pub fn delete_page(c : &Connection, page : Page){match c.execute("DELETE FROM page WHERE page_id = ?1", params![page.page_id]){
    Err(e)=>{println!("Could not create User with that arguments! | {}", e)}
    Ok(_)=>{
        let mut page_dir = utils::get_program_files_location();
        page_dir.push_str(PAGE_DIR);
        page_dir.push_str(&page.page_id);
        println!("{}", page_dir);
        std::fs::remove_dir_all(page_dir);
    }
}}


pub fn set_page_admin(c : &Connection, page_id : &str, user_id: &str){
    match c.execute(
        "INSERT INTO page_user (page_id, user_id) VALUES (?1, ?2)", (page_id, user_id))
    {
        Ok(_)=>{}
        Err(e)=>{println!("Could not create page_user with that arguments! | {}", e)}
    }
}

pub fn is_page_admin(c : &Connection, page_id : &str, user_id: &str) -> bool{
    match c.query_row(
        "SELECT page_id FROM page_user WHERE page_id = ?1 AND user_id = ?2",
        [page_id, user_id],
        |row| row.get(0),) as Result<String, rusqlite::Error>{
        Ok(_)=> true,
        Err(_)=>false
    }
}