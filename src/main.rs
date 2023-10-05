//http://127.0.0.1:8000/p/TH/blocodenotasfvcm
//mod cors;
//mod sessions;

//#[path ="database_handlers/pages.rs"]
//mod content;
//mod language_file;
mod authentication;
mod configuration;
mod editor;
pub mod pages;
pub mod users;

mod catchers;

use pages::Permission;
use rocket::config::LogLevel;
use rocket::fairing::{self, AdHoc};
use rocket::figment::providers::{Env, Format, Serialized, Toml};
use rocket::fs::NamedFile;
use rocket::response::status::NotFound;
use rocket::{catchers, get, tokio, Build, Rocket};
use rocket::{
    fs::{relative, FileServer},
    launch, routes,
};
use rocket_dyn_templates::Template;
use rocket_session_store::{self, memory::MemoryStore, CookieConfig, SessionStore};
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;
use users::user;
//use users::user::{ User, self};
use rocket_db_pools::{sqlx, Connection, Database};
use std::convert::Infallible;

use rocket::{
    http::HeaderMap,
    outcome::Outcome,
    request::{self, FromRequest},
    Request,
};

pub struct RequestHeaders<'h>(pub &'h HeaderMap<'h>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RequestHeaders<'r> {
    type Error = Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let request_headers = request.headers();
        Outcome::Success(RequestHeaders(request_headers))
    }
}

#[derive(Database)]
#[database("sqlite_logs")]
pub struct DATABASE(pub sqlx::SqlitePool);

async fn database_startup(rocket: Rocket<Build>) -> fairing::Result {
    if let Some(db) = DATABASE::fetch(&rocket) {
        sqlx::query(include_str!("../sqlite/startup.sql"))
            .execute(&db.0)
            .await
            .unwrap();
        let connection = &mut db.0.acquire().await.unwrap();

        let user = user::User::new(
            connection,
            "dayos".to_owned(),
            "1234".to_owned(),
            "daniela.paladinof@gmail.com".to_owned(),
            user::UserAccountStatus::Normal,
        )
        .await
        .unwrap();
        let page = pages::page::Page::new(connection, "page_debug".to_string(), None)
            .await
            .unwrap();
        page.set_collaborator(connection, &user, Permission::Owner)
            .await
            .unwrap();
        Ok(rocket)
    } else {
        Err(rocket)
    }
}

#[launch]
fn rocket() -> _ {
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
            File::create("my_rust_binary.log").unwrap(),
        ),
    ])
    .unwrap();

    //REMOVE AFTER DEBUG VVVV
    _ = std::fs::remove_file("/home/ubuntu/DEV/daytheipc-com/data/db.sqlite");
    _ = std::fs::remove_dir_all("/home/ubuntu/DEV/daytheipc-com/data/page_debug/");
    //^^^

    let store: SessionStore<String> = SessionStore {
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
    rocket::custom(figment)
        .register("/", catchers![catchers::not_found, catchers::not_authorized, catchers::internal_error])
        //built in no tworking
        //.mount("/static", FileServer::from(relative!("static")))
        .mount(
            "/",
            routes![
                get_file,
                pages::dir::get_dir_contents,
                pages::get_file,
                authentication::page,
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
        .attach(Template::fairing())
}

//.manage(authentication::AuthBuffer::new())
//NOT WORKING BECAUSE IT CONFLICTS WITH page::get_file
#[get("/static/<path..>")]
pub async fn get_file(path: PathBuf) -> Result<NamedFile, NotFound<String>> {
    let mut _path = PathBuf::from("static");
    _path.push(path);
    NamedFile::open(_path)
        .await
        .map_err(|_: std::io::Error| NotFound("File not found".into()))
}
