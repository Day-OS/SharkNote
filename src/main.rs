//http://127.0.0.1:8000/p/TH/blocodenotasfvcm
//mod cors;
mod table_of_contents;
//#[path ="database_handlers/pages.rs"]
mod pages;
mod user;
mod utils;
mod webeditor;
mod content;
mod language_file;
//mod sessions;


use std::{sync::Mutex, fs::File};
use std::time::Duration;
use pages::Page;
use rocket::{routes, launch, fs::{FileServer, relative}};
use rocket_session_store::{self, SessionStore, CookieConfig};
use rocket::config::{LogLevel};
use rocket_session_store::memory::MemoryStore;
use rocket_dyn_templates::{Template};
use rusqlite::Connection;

type DbConn = Mutex<Connection>;



#[launch]
fn rocket() -> _ {
    _ = std::fs::remove_file("/home/ubuntu/daytheipc-com/db/database.db");
    let connection = utils::get_database(vec![user::CREATION_RULES, pages::PAGE_CREATION_RULES, pages::PAGE_TRIGGER_RULES, pages::PAGEUSER_CREATION_RULES]);
    user::create_user(
        &connection, 
        user::User{
            user_id: "dayos".to_string(), 
            password: "1234".to_string(), 
            display_name: None,
            description: None, 
            profile_picture: None,
            is_program_admin: 1,
        }
    );
    pages::create_page(&connection, "seggs".into());
    pages::set_page_admin(&connection, "seggs", "dayos");
    pages::create_page(&connection, "seggs2".into());
    pages::set_page_admin(&connection, "seggs2", "dayos");
    


    

	let store: SessionStore<String> = SessionStore {
		store: Box::new(MemoryStore::default()),
		name: "token".into(),
		duration: Duration::from_secs(3600 * 24 * 3),
		// The cookie config is used to set the cookie's path and other options.
		cookie: CookieConfig{path:Some("/".to_string()), ..Default::default()}
	};
    let config = utils::get_config_file();
    let figment = rocket::Config::figment()
        .merge(("secret_key", config.secret_key))
        .merge(("port", config.port))
        .merge(("log_level", LogLevel::Critical));
    rocket::custom(figment).mount("/", routes![
        webeditor::post_editor,
        webeditor::editor,
        webeditor::create_page,
        content::get_content
    ])
    .mount("/static", FileServer::from(relative!("static")))
    .manage(Mutex::new(connection))
    .attach(store.fairing())
    .attach(Template::fairing())
}