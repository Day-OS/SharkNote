use std::path::PathBuf;
use rocket_dyn_templates::{Template, context};
use rocket::{
    get,
    http::Status,
    tokio::fs, State,
};
static PROGRAM_NAME: &str = "daytheipc-com";
use crate::{configuration, pages::permissions};
use rocket_db_pools::Connection;

use rocket_session_store::Session;

#[get("/editor/components/explorer?<page_id>&<path>")]
pub async fn get_dir_contents(
    mut connection: Connection<crate::DATABASE>,
    page_id: String,
    path: String,
    session: Session<'_, String>,
    config: &State<configuration::SharkNoteConfig>,
) -> Result<Template, Status> {
    let page = permissions::get_page_if_allowed(&mut connection, 
        &page_id,
        &session, 
        Some(vec!(permissions::Permission::ModifyContent)))
    .await?;
    println!("data/{}/{}", page_id, path);
    let mut _path = PathBuf::from(format!("data/{}/{}", page_id, path));
    let path_name = _path.to_str().unwrap().to_owned();
    
    let mut directories: Vec<String> = vec!();
    let mut files: Vec<String> = vec!();


    let mut dir: fs::ReadDir = fs::read_dir(_path)
    .await
    .map_err(|_| Status::NotFound)?;
    while let Some(child) = dir
        .next_entry()
        .await
        .map_err(|_| Status::InternalServerError)?
    {
        let metadata = child.metadata().await.unwrap();
        if metadata.is_dir()       { directories.push(child.file_name().into_string().unwrap()); } 
        else if metadata.is_file() { files.push(child.file_name().into_string().unwrap()); }
    }

    Ok(Template::render("editor/components/file_explorer", context!{
        path: path_name,
        messages: config.messages.clone(),
        directories: directories,
        file: files,
    }))
}