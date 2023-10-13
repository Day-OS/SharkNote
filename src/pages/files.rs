use std::{path::{PathBuf, Component}, str::FromStr};
use log::info;
use rocket_multipart_form_data::{MultipartFormDataOptions, MultipartFormDataField, mime, MultipartFormData};
use strum::EnumString;
use rocket::{
    futures::{future::BoxFuture, FutureExt},
    get,
    http::{Status, ContentType},
    serde::json::Json,
    tokio::fs, fs::NamedFile, FromForm, post, form::Form, shield::Permission, response::content::RawText,
};
static PROGRAM_NAME: &str = "daytheipc-com";
use crate::{authentication::SessionCookie, pages::page, users::user};
use rocket_db_pools::Connection;

use rocket_session_store::Session;
use serde::{Serialize, Deserialize};
use super::page::Page;
use base64::{self, Engine, engine::{self, general_purpose}};
use rocket::Data;

// ----------------------------------------------------------------------------//
// File getting

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

async fn get_page_for_viewing(connection: &mut Connection<crate::DATABASE>,
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
                page.get_user_permission(&mut *connection, user)
                    .await
                    .map_err(|_| Status::Unauthorized)?;
                //The permission is not checked because it doesn't 
                //matter if the user is an admin or someone who was just invited to view the page.
                //If anyone has a permission to this page, they can see it.
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
    let page = get_page_for_viewing(&mut connection, &page_id, &session).await?;

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
    let page = get_page_for_viewing(&mut connection, &page_id, &session).await?;

    let mut _path = PathBuf::from("data");
    _path.push(page.page_id.clone());

    recursive_search(_path, Directory::default())
        .await
        .map(|dir| Json(dir))
        .map_err(|_| Status::InternalServerError)
}

// ----------------------------------------------------------------------------//
// File alteration

/// This checks if the user has the permission to alter anything to the page.
/// In the case the user is a 'colaborator', then it will return the results,
/// otherwhise it will return an error.
/// 
/// Permission will always be returned as an owner or an admin, nothing more.
async fn get_page_and_permission_for_alteration(connection: &mut Connection<crate::DATABASE>,
    page_id: &String,
    session: &Session<'_, String>) -> Result<(Page, super::Permission), Status> {
    let page = page::Page::get(&mut *connection, page_id.to_string())
    .await
    .map_err(|_| Status::NotFound)?;
    if let SessionCookie::LoggedIn { user_id } = SessionCookie::get(&session).await {
        let user = user::User::get(&mut *connection, user_id).await.unwrap();
        let perm = page
            .get_user_permission(&mut *connection, user)
            .await
            .map_err(|_| Status::InternalServerError)?;
        match perm {
            //Only those two can edit files. The permission is sent back to the function caller because
            //this function is used both for file alteration and for the page itself.
            super::Permission::Owner |  super::Permission::Admin  => {
                Ok((page, perm))
            },
            _ =>{
                return Err(Status::Unauthorized);
            }
        }
    } else {
        return Err(Status::Unauthorized);
    }
}

#[derive(EnumString)]
enum Type {
    #[strum(serialize = "file")]
    File,
    #[strum(serialize = "dir")]
    Dir,
}



#[post("/editor/write/<page_id>", data = "<data>")]
pub async fn write_file(
    mut connection: Connection<crate::DATABASE>,
    content_type: &ContentType, 
    session: Session<'_, String>,
    page_id: String,
    data: Data<'_>,
) -> Result<(), Status> {
    //permission is discarded in this case because both the Owner and Admin role has the permission to create a file. 
    let _ = get_page_and_permission_for_alteration(&mut connection, &page_id, &session).await?;

    let mut options = MultipartFormDataOptions::with_multipart_form_data_fields(
        vec! [
            MultipartFormDataField::file("content").size_limit(32 * 1024 * 1024),
            MultipartFormDataField::text("req_type"),
            MultipartFormDataField::text("path"),
        ]
    );

    let mut multipart_form_data = MultipartFormData::parse(content_type, data, options).await.unwrap();
    let req_type = multipart_form_data.texts.remove("req_type").unwrap().first().unwrap().text.clone();
    let _path = multipart_form_data.texts.remove("path").unwrap().first().unwrap().text.clone();
    let mut path = PathBuf::from(format!("data/{}{}", page_id, _path));
    
    match Type::from_str(&req_type).map_err(|_| Status::BadRequest)?{
        Type::File =>{
            let content = multipart_form_data.files.remove("content");
            for file in content.unwrap() {
                path.push(file.file_name.unwrap());
                if path.components().into_iter().any(|x| x == Component::ParentDir) {
                    return Err(Status::Unauthorized);
                }
                let tmp_file = fs::read(file.path).await.unwrap();

                match fs::File::create(path.clone()).await {
                    Ok(_) => info!("{path:?} created!"),
                    Err(_) => info!("{path:?} already exists or could not be created!"),
                }
                fs::write(path.clone(), tmp_file).await.map_err(|e| {println!("{e}");Status::InternalServerError})?;
                println!("{:?}", path);
            }
        } 
        Type::Dir => fs::create_dir_all(path).await.map_err(|_| Status::InternalServerError)?,
    }


    
    return Ok(());






    /*
    
    let path = PathBuf::from(format!("data/{}{}", page_id, json.path.clone()));
    
    if path.components().into_iter().any(|x| x == Component::ParentDir) {
        return Err(Status::Unauthorized);
    }
    
    Ok(())
    */
}