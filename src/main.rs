//http://127.0.0.1:8000/p/TH/blocodenotasfvcm
//mod cors;
mod table_of_contents;
//#[path ="database_handlers/pages.rs"]
//mod pages;
#[path ="database_handlers/page_owner.rs"]
mod page_owner;
#[path ="database_handlers/db_utils.rs"]
mod db_utils;
mod webeditor;
mod config_file;
mod language_file;
//mod sessions;


use std::sync::Mutex;
use std::time::Duration;
use rocket::{routes, launch};
use rocket_session_store::{self, SessionStore, CookieConfig};
use rocket::config::{LogLevel};
use rocket_session_store::memory::MemoryStore;
use rocket_dyn_templates::{Template};
use rusqlite::Connection;

type DbConn = Mutex<Connection>;


/*
    fn from_request<'life0,'async_trait>(request: &'r rocket::Request<'life0>) ->  core::pin::Pin<Box<dyn core::future::Future<Output = request::Outcome<Self,Self::Error> > + core::marker::Send+'async_trait> >where 'r:'async_trait,'life0:'async_trait,Self:'async_trait {
        todo!()
    } */


#[launch]
fn rocket() -> _ {
    let pageowner_conn = page_owner::get_database();
    //page_owner::delete_user(&pageowner_conn, "dayos");
    page_owner::create_user(
        &pageowner_conn, 
        page_owner::PageOwner{
            page_owner_id: "dayos".to_string(), 
            password: "1234".to_string(), 
            display_name: None,
            description: None, 
            profile_picture: None,
            is_program_admin: 1
        }
    );
    

    let memory_store: MemoryStore::<String> = MemoryStore::default();
	let store: SessionStore<String> = SessionStore {
		store: Box::new(memory_store),
		name: "token".into(),
		duration: Duration::from_secs(3600 * 24 * 3),
		// The cookie config is used to set the cookie's path and other options.
		cookie: CookieConfig{path:Some("/".to_string()), ..Default::default()}
	};
    /* = Databases{
        page_owners_db: page_owner::get_database(),
        pages: vec![]
    }; */
    let config = config_file::get_config_file();
    let figment = rocket::Config::figment()
        .merge(("secret_key", config.secret_key))
        .merge(("port", config.port))
        .merge(("log_level", LogLevel::Critical));
    rocket::custom(figment).mount("/", routes![
        //oldgetdocument::get_page, 
        //::get_content, 
        webeditor::post_editor,
        webeditor::editor, 
    ])
    .manage(Mutex::new(pageowner_conn))
    .attach(store.fairing())
    .attach(Template::fairing())
    //.attach(cors.)
}