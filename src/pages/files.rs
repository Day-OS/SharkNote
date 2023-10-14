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
use rocket_db_pools::Connection;

use rocket_session_store::Session;
use super::permissions;
use rocket::Data;


// ----------------------------------------------------------------------------//
// File getting





#[get("/<page_id>/<path..>")]
pub async fn get_file(
    mut connection: Connection<crate::DATABASE>,
    page_id: String,
    path: PathBuf,
    session: Session<'_, String>,
) -> Result<NamedFile, Status> {
    let page = 
        permissions::get_page_if_allowed(&mut connection, 
            &page_id,
            &session, 
            Some(vec!(permissions::Permission::SeePrivate)))
        .await?;

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
    let _ = permissions::get_page_if_allowed(&mut connection, 
        &page_id,
        &session, 
        Some(vec!(permissions::Permission::ModifyContent)))
    .await?;
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
}