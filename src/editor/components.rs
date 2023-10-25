use crate::authentication::{SessionToken, CSRF};
use crate::pages::files;
use crate::{configuration, pages::permissions};
use rocket::{form::Form, get, http::Status, post, tokio::fs, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::{context, Template};
use serde::{Serialize, Deserialize};
use std::default;
use std::hash::Hash;
use std::{collections::HashMap, path::PathBuf};

use rocket_session_store::Session;

#[post("/editor/components/deletion_modal", data = "<data>")]
pub async fn deletion_modal(
    mut connection: Connection<crate::DATABASE>,
    session: Session<'_, SessionToken>,
    data: Form<HashMap<String, String>>,
    config: &State<configuration::SharkNoteConfig>,
    csrf: &State<CSRF>,
) -> Result<Template, Status> {
    if let (Some(page_id), Some(path)) = (data.get("page_id"), data.get("path")) {
        let _page: crate::pages::Page = permissions::get_page_if_allowed(
            &mut connection,
            &page_id,
            &session,
            vec![permissions::Permission::ModifyContent],
            csrf

        )
        .await?;
        let path_name = files::check_path_traversal_attack(PathBuf::from(path.clone()))?
            .to_str()
            .unwrap()
            .to_owned();

        return Ok(Template::render(
            "editor/components/deletion_modal",
            context! {
                page_id: page_id,
                path: path_name,
                messages: config.messages.clone(),
            },
        ));
    }
    Err(Status::BadRequest)
}

#[post("/editor/components/renaming_modal", data = "<data>")]
pub async fn renaming_modal(
    mut connection: Connection<crate::DATABASE>,
    session: Session<'_, SessionToken>,
    data: Form<HashMap<String, String>>,
    config: &State<configuration::SharkNoteConfig>,
    csrf: &State<CSRF>,
) -> Result<Template, Status> {
    if let (Some(page_id), Some(path),  Some(_type)) = (data.get("page_id"), data.get("path"), data.get("_type")) {
        let _page: crate::pages::Page = permissions::get_page_if_allowed(
            &mut connection,
            &page_id,
            &session,
            vec![permissions::Permission::ModifyContent],
            csrf
        )
        .await?;
    
        return Ok(Template::render(
            "editor/components/renaming",
            context! {
                _type: _type,
                page_id: page_id,
                path: path,
                messages: config.messages.clone(),
            },
        ));
    }
    Err(Status::BadRequest)
}


#[post("/editor/components/dir_creation_modal", data = "<data>")]
pub async fn dir_creation_modal(
    mut connection: Connection<crate::DATABASE>,
    session: Session<'_, SessionToken>,
    data: Form<HashMap<String, String>>,
    config: &State<configuration::SharkNoteConfig>,
    csrf: &State<CSRF>,
) -> Result<Template, Status> {
    if let (Some(page_id), Some(path)) = (data.get("page_id"), data.get("path")) {
        let _page: crate::pages::Page = permissions::get_page_if_allowed(
            &mut connection,
            &page_id,
            &session,
            vec![permissions::Permission::ModifyContent],
            csrf
        )
        .await?;
        let path_name = files::check_path_traversal_attack(PathBuf::from(path.clone()))?
            .to_str()
            .unwrap()
            .to_owned();

        return Ok(Template::render(
            "editor/components/dir_creation_modal",
            context! {
                page_id: page_id,
                path: path_name,
                messages: config.messages.clone(),
            },
        ));
    }
    Err(Status::BadRequest)
}

#[post("/editor/components/file_creation_modal", data = "<data>")]
pub async fn file_creation_modal(
    mut connection: Connection<crate::DATABASE>,
    session: Session<'_, SessionToken>,
    data: Form<HashMap<String, String>>,
    config: &State<configuration::SharkNoteConfig>,
    csrf: &State<CSRF>,
) -> Result<Template, Status> {
    if let (Some(page_id), Some(path)) = (data.get("page_id"), data.get("path")) {
        let _page: crate::pages::Page = permissions::get_page_if_allowed(
            &mut connection,
            &page_id,
            &session,
            vec![permissions::Permission::ModifyContent],
            csrf
        )
        .await?;
        let path_name = files::check_path_traversal_attack(PathBuf::from(path.clone()))?
            .to_str()
            .unwrap()
            .to_owned();

        return Ok(Template::render(
            "editor/components/file_creation_modal",
            context! {
                page_id: page_id,
                path: path_name,
                messages: config.messages.clone(),
            },
        ));
    }
    Err(Status::BadRequest)
}





#[get("/editor/components/explorer?<page_id>&<path>")]
pub async fn explorer(
    mut connection: Connection<crate::DATABASE>,
    page_id: String,
    path: String,
    session: Session<'_, SessionToken>,
    config: &State<configuration::SharkNoteConfig>,
    csrf: &State<CSRF>,
) -> Result<Template, Status> {
    let _page = permissions::get_page_if_allowed(
        &mut connection,
        &page_id,
        &session,
        vec![permissions::Permission::ModifyContent],
        csrf
    )
    .await?;

    let mut _path = PathBuf::from(path.clone());
    let path_name = _path.to_str().unwrap().to_owned();
    let parent_path_name = {
        let mut p: PathBuf = _path.clone();
        if p.pop() {
            Some(p.to_str().unwrap().to_owned())
        } else {
            None
        }
    };

    let mut directories: Vec<String> = vec![];
    let mut files: Vec<String> = vec![];

    let mut pathbuf = PathBuf::from("data");
    pathbuf.push(page_id.clone());
    pathbuf.push(path);

    let mut dir: fs::ReadDir = fs::read_dir(pathbuf).await.map_err(|_| Status::NotFound)?;
    while let Some(child) = dir
        .next_entry()
        .await
        .map_err(|_| Status::InternalServerError)?
    {
        let metadata = child.metadata().await.unwrap();
        if metadata.is_dir() {
            directories.push(child.file_name().into_string().unwrap());
        } else if metadata.is_file() {
            files.push(child.file_name().into_string().unwrap());
        }
    }

    Ok(Template::render(
        "editor/components/explorer",
        context! {
            page_id: page_id,
            parent_path: parent_path_name,
            path: path_name,
            messages: config.messages.clone(),
            directories: directories,
            files: files,
        },
    ))
}





#[post("/editor/components/notification", data = "<data>")]
pub async fn notification(
    data: Form<HashMap<String, String>>,
) -> Result<Template, Status> {
    return Ok(Template::render(
        "editor/components/notification", super::Notification::from(&data)
    ));
}