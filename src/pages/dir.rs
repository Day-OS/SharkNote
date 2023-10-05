use std::path::PathBuf;

use dirs;
use filetime::FileTime;
use inquire::Password;
use rocket::{futures::{TryFutureExt, future::BoxFuture, FutureExt}, tokio::fs, serde::json::Json, response::status::NotFound, get, http::Status};
static PROGRAM_NAME: &str = "daytheipc-com";
use rocket_db_pools::Connection;
use rocket_dyn_templates::{Template, context};
use rocket_session_store::Session;
use serde::{Deserialize, Serialize};
use crate::{pages::page, authentication::SessionCookie, users::user};

use super::page::Page;

pub fn get_program_files_location() -> String {
    format!(
        "{}/{}",
        dirs::home_dir().unwrap().to_str().unwrap().to_string(),
        PROGRAM_NAME
    )
}

#[derive(Serialize)]
pub struct Directory {
    pub(crate) name: String,
    pub(crate) directories: Vec<Directory>,
    pub(crate) files: Vec<File>,
}
impl Default for Directory {
    fn default() -> Self {
        Self {
            name: "".into(),
            directories: vec![],
            files: vec![],
        }
    }
}

#[derive(Serialize)]
pub struct File {
    pub(crate) name: String,
}

fn recursive_search<'a>(path: PathBuf, mut root_dir: Directory) -> BoxFuture<'a, Result<Directory, String>> {
    async {
        let mut dir: fs::ReadDir = fs::read_dir(path).await.map_err(|_|{"Not found!".to_string()})?;
        while let Some(child) = dir
            .next_entry()
            .await
            .map_err(|_|{"Could not read dir".to_string()})?{
            let metadata = child.metadata().await.unwrap();
            if metadata.is_dir() {
                root_dir.directories.push(
                    recursive_search(
                        child.path(),
                        Directory {
                            name: child.file_name().into_string().unwrap(),
                            ..Default::default()
                        },
                    )
                    .await?,
                );
            } else if metadata.is_file() {
                root_dir.files.push(File {
                    name: child.file_name().into_string().unwrap()
                });
            }
        }
        Ok(root_dir)
    }.boxed()
}

#[get("/dir/<page_id>")]
pub async fn get_dir_contents(
    mut connection: Connection<crate::DATABASE>,
    page_id: String,
    session: Session<'_, String>,
) -> Result<Json<Directory>, Status> {
    let page = page::Page::get(&mut *connection, page_id)
        .await
        .map_err(|_: sqlx::Error| Status::NotFound)?;
    match page.status {
        super::PageStatus::Public | super::PageStatus::LinkOnly => {},
        super::PageStatus::Private => {
            if let SessionCookie::LoggedIn { user_id } = SessionCookie::get(&session).await {
                let user = user::User::get(&mut *connection, user_id).await.unwrap();
                let result = page.check_if_user_is_colaborator(&mut *connection, user).await.map_err(|_| Status::InternalServerError)?;
                if result == false {
                    return Err(Status::Unauthorized)
                }
            }
            else{
                return Err(Status::Unauthorized)
            }
        },
    }

    let mut _path = PathBuf::from("data");
    _path.push(page.page_id.clone());

    recursive_search(_path, Directory::default())
    .await
    .map(|dir| Json(dir))
    .map_err(|_|{Status::InternalServerError})
    
}
