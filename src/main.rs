//http://127.0.0.1:8000/p/TH/blocodenotasfvcm
//mod cors;
//mod sessions;

//#[path ="database_handlers/pages.rs"]
//mod content;
//mod language_file;
#![feature(async_closure)]
mod authentication;
mod catchers;
mod configuration;
mod editor;
pub mod pages;
pub mod users;

use csrf::AesGcmCsrfProtection;
use log::warn;
use pages::permissions::Permission;
use rocket::config::{LogLevel, SecretKey};
use rocket::fairing::{self, AdHoc};
use rocket::figment::providers::{Env, Format, Serialized, Toml};
use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::response::status::NotFound;
use rocket::serde::json::to_value;
use rocket::serde::ser::StdError;
use rocket::{catchers, get, Build, Rocket};
use rocket::{launch, routes};
use rocket_dyn_templates::Template;
use rocket_session_store::{self, memory::MemoryStore, CookieConfig, SessionStore};
use serde_json::Value;
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;
use users::{User, UserAccountStatus};

//use users::user::{ User, self};
use rocket_db_pools::{sqlx, Database};
use std::convert::Infallible;

use rocket::{
    http::HeaderMap,
    outcome::Outcome,
    request::{self, FromRequest},
    Request,
};

use crate::authentication::{SessionToken, CSRF};

/*
pub struct RequestHeaders<'h>(pub &'h HeaderMap<'h>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RequestHeaders<'r> {
    type Error = Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let request_headers = request.headers();
        Outcome::Success(RequestHeaders(request_headers))
    }
}
 */

#[derive(Database)]
#[database("sqlite_logs")]
pub struct DATABASE(pub sqlx::SqlitePool);


pub fn none(
    value: Option<&Value>,
    _args: &[Value],
) -> Result<bool, rocket_dyn_templates::tera::Error> {
    Ok(value.unwrap().is_null())
}
pub fn extension(
    value: &Value,
    _: &HashMap<String, Value>,
) -> Result<Value, rocket_dyn_templates::tera::Error> {
    match value {
        Value::String(s) => Ok(to_value(s.split('.').last().unwrap().to_owned()).unwrap()),
        _ => Err(rocket_dyn_templates::tera::Error::msg(
            "Extension filter should only be used in strings",
        )),
    }
}
pub fn content_name(
    value: &Value,
    _: &HashMap<String, Value>,
) -> Result<Value, rocket_dyn_templates::tera::Error> {
    match value {
        Value::String(s) => Ok(to_value(s.split('/').last().unwrap().to_owned()).unwrap()),
        _ => Err(rocket_dyn_templates::tera::Error::msg(
            "CurrentPath filter should only be used in strings",
        )),
    }
}

async fn database_startup(rocket: Rocket<Build>) -> fairing::Result {
    if let Some(db) = DATABASE::fetch(&rocket) {
        sqlx::query(include_str!("../sqlite/startup.sql"))
            .execute(&db.0)
            .await
            .unwrap();
        let connection = &mut db.0.acquire().await.unwrap();

        let user = User::new(
            connection,
            "dayos".to_owned(),
            "1234".to_owned(),
            "daniela.paladinof@gmail.com".to_owned(),
            UserAccountStatus::Normal,
        )
        .await
        .unwrap();
        let page = pages::Page::new(connection, "page_debug".to_string(), None)
            .await
            .unwrap();
        page.set_permission(connection, &user, Permission::ModifyContent)
            .await
            .unwrap();
        page.set_permission(connection, &user, Permission::SeePrivate)
            .await
            .unwrap();
        Ok(rocket)
    } else {
        Err(rocket)
    }
}

#[launch]
fn rocket() -> _ {
    let tera = Template::custom(|engines| {
        engines.tera.register_tester("none", none);
        engines.tera.register_filter("extension", extension);
        engines.tera.register_filter("content_name", content_name);
        
    });
    if let Err(e) = std::process::Command::new("git")
        .args(["submodule", "update", "--init", "--recursive"])
        .output()
    {
        panic!("Something went wrong when downloading submodules! {}", e)
    };
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("shark_note.log").unwrap(),
        ),
    ])
    .unwrap();

    //REMOVE AFTER DEBUG VVVV
    _ = std::fs::remove_file("/home/ubuntu/DEV/daytheipc-com/data/db.sqlite");
    //_ = std::fs::remove_dir_all("/home/ubuntu/DEV/daytheipc-com/data/page_debug/");
    //^^^

    let store: SessionStore<SessionToken> = SessionStore {
        store: Box::new(MemoryStore::default()),
        name: "token".into(),
        duration: Duration::from_secs(3600 * 24 * 3),
        // The cookie config is used to set the cookie's path and other options.
        cookie: CookieConfig {
            path: Some("/".to_string()),
            ..Default::default()
        },
    };
    //let config = configuration::get_config_file();
    let figment = rocket::Config::figment()
        .merge(Serialized::defaults(
            configuration::SharkNoteConfig::default(),
        ))
        .merge(Env::prefixed("APP_").global())
        .merge(Toml::file("configuration.toml").nested())
        .merge(("log_level", LogLevel::Critical));


    let mut secret: [u8; 32] = [0; 32];
    secret.copy_from_slice(
        figment.find_value("secret_key")
        .unwrap()
        .as_str()
        .unwrap()
        .as_bytes().split_at(32).0
    );
    warn!("{secret:?}");

    rocket::custom(figment)
        .register(
            "/",
            catchers![
                catchers::not_found,
                catchers::not_authorized,
                catchers::internal_error
            ],
        )
        //built in no tworking
        //.mount("/static", FileServer::from(relative!("static")))
        .mount(
            "/",
            routes![
                get_file,
                pages::files::get_file,
                pages::files::dir_create,
                pages::files::write_file,
                pages::files::delete,
                pages::files::rename,
                editor::components::notification,
                editor::components::explorer,
                editor::components::file_creation_modal,
                editor::components::dir_creation_modal,
                editor::components::deletion_modal,
                editor::components::renaming_modal,
                authentication::base,
                authentication::logout::logout,
                authentication::main_forms::login_register_forms,
                authentication::password_reset::page,
                authentication::password_reset::post,
                authentication::password_reset::confirmation,
                authentication::login::post,
                authentication::login::confirmation,
                authentication::register::post,
                authentication::register::confirmation,
                authentication::register::check,
                editor::editor,
            ],
        )
        .attach(AdHoc::config::<configuration::SharkNoteConfig>())
        .attach(DATABASE::init())
        .attach(rocket_recaptcha_v3::ReCaptcha::fairing())
        .attach(AdHoc::try_on_ignite("Database Startup", database_startup))
        .attach(store.fairing())
        .attach(tera)
        .manage(CSRF(AesGcmCsrfProtection::from_key(secret)))
}

//.manage(authentication::AuthBuffer::new())
//NOT WORKING BECAUSE IT CONFLICTS WITH page::get_file
#[get("/static/<path..>")]
pub async fn get_file(path: PathBuf) -> Result<NamedFile, Status> {
    let mut _path = PathBuf::from("static");
    _path.push(path);
    NamedFile::open(_path)
        .await
        .map_err(|_: std::io::Error| Status::NotFound)
}
