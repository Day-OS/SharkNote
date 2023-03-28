//http://127.0.0.1:8000/p/TH/blocodenotasfvcm
//mod cors;
mod table_of_contents;
mod database_handler;
mod webeditor;
mod oldgetdocument;
#[macro_use] extern crate rocket;
use rocket::fs::NamedFile;
use rocket::response::content::{RawHtml};
use rocket::config::{LogLevel};
use rocket_auth::Users;
use rocket_dyn_templates::Template;
use std::borrow::BorrowMut;
use std::fs;
use std::str;
use html_editor::{operation::*, Node};
static DEFAULT_DIR: &'static str = "/home/ubuntu/.daytheipc-com/";

#[launch]
fn rocket() -> _ {
    let users = Users::open_rusqlite(database_handler::get_db_file_dir("admins")).unwrap();

    let figment = rocket::Config::figment()
        .merge(("secret_key", "4DE55BX8qx1m4SAEbmtUT9HXPHUHyrfRXl7JtjjTwWw="))
        .merge(("port", 8000))
        .merge(("log_level", LogLevel::Critical));
    rocket::custom(figment).mount("/", routes![
        oldgetdocument::get_page, 
        oldgetdocument::get_content, 
        webeditor::get_editor, 
    ]).manage(users)    
    .attach(Template::fairing())
    //.attach(cors.)
}

/*#[rocket::main]
 fn main() -> Result<(), rocket::Error>{
    let users = Users::open_rusqlite("mydb.db")?;
    //database_handler::create_table_if_not_exists("TH.db");

    let figment = rocket::Config::figment()
        .merge(("port", 8000))
        .merge(("log_level", LogLevel::Critical));
    rocket::custom(figment).mount("/", routes![get_page, get_content, webeditor::get_editor]);//.attach(cors.)
    Ok(())
}
 */