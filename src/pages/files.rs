use std::path::PathBuf;

use dirs;
use filetime::FileTime;
use inquire::Password;
use rocket::{
    futures::{future::BoxFuture, FutureExt, TryFutureExt},
    get,
    http::Status,
    response::status::NotFound,
    serde::json::Json,
    tokio::fs, fs::NamedFile, FromForm, post, form::Form,
};
static PROGRAM_NAME: &str = "daytheipc-com";
use crate::{authentication::{SessionCookie, login::post}, pages::page, users::user};
use rocket_db_pools::Connection;
use rocket_dyn_templates::{context, Template};
use rocket_session_store::Session;
use serde::{Deserialize, Serialize};
use super::page::Page;

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

fn recursive_search<'a>(
    path: PathBuf,
    mut root_dir: Directory,
) -> BoxFuture<'a, Result<Directory, String>> {
    async {
        let mut dir: fs::ReadDir = fs::read_dir(path)
            .await
            .map_err(|_| "Not found!".to_string())?;
        while let Some(child) = dir
            .next_entry()
            .await
            .map_err(|_| "Could not read dir".to_string())?
        {
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
                    name: child.file_name().into_string().unwrap(),
                });
            }
        }
        Ok(root_dir)
    }
    .boxed()
}
async fn get_page_if_user_is_allowed(connection: &mut Connection<crate::DATABASE>,
    page_id: &String,
    session: &Session<'_, String>) -> Result<Page, Status> {
    let page = page::Page::get(&mut *connection, page_id.to_string())
    .await
    .map_err(|_| Status::NotFound)?;
    match page.status {
        super::PageStatus::Public | super::PageStatus::LinkOnly => {}
        super::PageStatus::Private => {
            if let SessionCookie::LoggedIn { user_id } = SessionCookie::get(&session).await {
                let user = user::User::get(&mut *connection, user_id).await.unwrap();
                let result = page
                    .check_if_user_is_colaborator(&mut *connection, user)
                    .await
                    .map_err(|_| Status::InternalServerError)?;
                if result == false {
                    return Err(Status::Unauthorized);
                }
            } else {
                return Err(Status::Unauthorized);
            }
        }
    }
    Ok(page)
}

#[get("/<page_id>/<path..>")]
pub async fn get_file(
    mut connection: Connection<crate::DATABASE>,
    page_id: String,
    path: PathBuf,
    session: Session<'_, String>,
) -> Result<NamedFile, Status> {
    let page = get_page_if_user_is_allowed(&mut connection, &page_id, &session).await?;

    let mut _path = PathBuf::from("data");
    _path.push(page.page_id.clone());
    _path.push(path);
    if _path.extension().is_none(){
        return Err(Status::NotImplemented);
    }

    NamedFile::open(_path)
        .await
        .map_err(|_: std::io::Error| Status::NotFound)
}


#[get("/dir/<page_id>")]
pub async fn get_dir_contents(
    mut connection: Connection<crate::DATABASE>,
    page_id: String,
    session: Session<'_, String>,
) -> Result<Json<Directory>, Status> {
    let page = get_page_if_user_is_allowed(&mut connection, &page_id, &session).await?;

    let mut _path = PathBuf::from("data");
    _path.push(page.page_id.clone());

    recursive_search(_path, Directory::default())
        .await
        .map(|dir| Json(dir))
        .map_err(|_| Status::InternalServerError)
}


#[derive(FromForm)]
pub struct CreationReq {
    pagename: String,
}


#[post("/editor/create", data = "<form>")]
pub async fn create_page(
    session: Session<'_, String>,
    form: Form<CreationReq>,
    connection: Connection<crate::DATABASE>,
) -> Result<(), Status> {
    let page = page::Page::get(&mut *connection, page_id.to_string())
        .await
        .map_err(|_| Status::NotFound)?;
    get_page_if_user_is_allowed(&mut connection, &page, &session).await?;


}