use log::{info, warn};
use rocket::{
    form::Form,
    fs::NamedFile,
    get,
    http::{ContentType, Status},
    post, put,
    tokio::fs,
    FromForm, delete, State,
};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use rocket_multipart_form_data::{
    MultipartFormData, MultipartFormDataField, MultipartFormDataOptions, TextField,
};
use std::{
    collections::HashMap,
    default,
    fmt::format,
    path::{Component, PathBuf},
    str::FromStr,
};

use crate::{editor::Notification, authentication::{SessionToken, CSRF}};

use super::permissions;
use rocket::Data;
use rocket_session_store::Session;

pub fn check_path_traversal_attack(path: PathBuf) -> Result<PathBuf, Status> {
    if path
        .components()
        .into_iter()
        .any(|x| x == Component::ParentDir)
    {
        warn!("Path traversal attack detected at {:?}", path);
        return Err(Status::Unauthorized);
    }
    Ok(path)
}

#[get("/<page_id>/<path..>")]
pub async fn get_file(
    mut connection: Connection<crate::DATABASE>,
    page_id: String,
    path: PathBuf,
    csrf: &State<CSRF>,
    session: Session<'_, SessionToken>,
) -> Result<NamedFile, Status> {
    let page = permissions::get_page_if_allowed(
        &mut connection,
        &page_id,
        &session,
        vec![permissions::Permission::SeePrivate],
        csrf
    )
    .await?;

    let mut _path = PathBuf::from("data");
    _path.push(page.page_id.clone());
    _path.push(path);
    let path = check_path_traversal_attack(_path)?;

    if path.extension().is_none() {
        return Err(Status::NotImplemented);
    }

    NamedFile::open(path)
        .await
        .map_err(|_: std::io::Error| Status::NotFound)
}
///Route that creates/update a file in the specified path and returns a Notification Component
#[put("/editor/file-write", data = "<data>", format = "multipart/form-data")]
pub async fn write_file(
    mut connection: Connection<crate::DATABASE>,
    session: Session<'_, SessionToken>,
    data: Data<'_>,
    content_type: &ContentType,
    csrf: &State<CSRF>,
) -> Result<Template, Template> {
    async move || -> Result<(), Status> {
        let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
            MultipartFormDataField::file("content").size_limit(32 * 1024 * 1024),
            MultipartFormDataField::text("path"),
            MultipartFormDataField::text("page_id"),
            MultipartFormDataField::text("file_name"),
        ]);

        let mut multipart_form_data = MultipartFormData::parse(content_type, data, options)
            .await
            .unwrap();

        let texts = &mut multipart_form_data.texts;

        if let (Some(path), Some(page_id)) = (texts.remove("path"), texts.remove("page_id")) {
            let (_path, page_id) = (
                path.first()
                    .unwrap_or(&TextField {
                        text: "".into(),
                        content_type: None,
                        file_name: None,
                    })
                    .text
                    .clone(),
                page_id.first().ok_or(Status::BadRequest)?.text.clone(),
            );
            let _ = permissions::get_page_if_allowed(
                &mut connection,
                &page_id,
                &session,
                vec![permissions::Permission::ModifyContent],
                csrf
            )
            .await?;

            let mut path = PathBuf::from("data");
            path.push(page_id.clone());
            path.push(_path);

            if let Some(file_name) = texts.remove("file_name") {
                if let Some(file_name) = file_name.first() {
                    let file_name = file_name.text.clone();
                    if !file_name.is_empty() {
                        let mut tmp_path = path.clone();
                        tmp_path.push(file_name);

                        fs::write(tmp_path, vec![])
                            .await
                            .map_err(|e| Status::InternalServerError)?;
                        return Ok(());
                    }
                }
            }
            if let Some(content) = multipart_form_data.files.remove("content") {
                for file in content {
                    let mut tmp_path = path.clone();
                    tmp_path.push(file.file_name.unwrap());
                    tmp_path = check_path_traversal_attack(tmp_path)?;

                    //Reads the content inside the Temporary file uploaded and transfers it to another place.
                    let tmp_file = fs::read(file.path).await.unwrap();
                    match fs::File::create(tmp_path.clone()).await {
                        Ok(_) => info!("{path:?} created!"),
                        Err(_) => info!("{path:?} already exists or could not be created!"),
                    }
                    fs::write(tmp_path, tmp_file)
                        .await
                        .map_err(|e| Status::InternalServerError)?;
                }
                return Ok(());
            }
        }

        return Err(Status::BadRequest);
    }()
    .await
    .map(|_| {
        Template::render(
            "editor/components/notification",
            Notification {
                title: "Success!".into(),
                message: "File created.".into(),
                ..Default::default()
            },
        )
    })
    .map_err(|e| {
        println!("{e}");
        Template::render(
            "editor/components/notification",
            Notification {
                title: "An error has occurred!".into(),
                message: format!("Error code: {}", e.code),
                ..Default::default()
            },
        )
    })
}


///Route that creates a directory in the specified path and returns a Notification Component
#[put("/editor/dir-create", data = "<data>")]
pub async fn dir_create(
    mut connection: Connection<crate::DATABASE>,
    session: Session<'_, SessionToken>,
    data: Form<HashMap<String, String>>,
    csrf: &State<CSRF>,
) -> Result<Template, Template>{
    async move || -> Result<(), Status> {
        if let (Some(directory_name), Some(page_id), Some(path)) = (
            data.get("directory_name"),
            data.get("page_id"),
            data.get("path"),
        ) {
            let _ = permissions::get_page_if_allowed(
                &mut connection,
                &page_id,
                &session,
                vec![permissions::Permission::ModifyContent],
                csrf
            )
            .await?;
    
            let mut _path = PathBuf::from("data");
            _path.push(page_id);
            _path.push(path);
            _path.push(directory_name);
    
            fs::create_dir_all(check_path_traversal_attack(_path)?)
                .await
                .map_err(|_| Status::InternalServerError)?;
            return Ok(());
        }
        Err(Status::BadRequest)
    }()
    .await
    .map(|_| {
        Template::render(
            "editor/components/notification",
            Notification {
                title: "Success!".into(),
                message: "Directory created.".into(),
                ..Default::default()
            },
        )
    })
    .map_err(|e| {
        println!("{e}");
        Template::render(
            "editor/components/notification",
            Notification {
                title: "An error has occurred!".into(),
                message: format!("Error code: {}", e.code),
                ..Default::default()
            },
        )
    })
}

///Route that deletes a content from a path in the specified path and returns a Notification Component
#[delete("/editor/delete", data = "<data>")]
pub async fn delete(
    mut connection: Connection<crate::DATABASE>,
    session: Session<'_, SessionToken>,
    data: Form<HashMap<String, String>>,
    csrf: &State<CSRF>,
) -> Result<Template, Template>{
    async move || -> Result<String, Status> {
        if let (Some(page_id), Some(path)) = (
            data.get("page_id"),
            data.get("path"),
        ) {
            let _ = permissions::get_page_if_allowed(
                &mut connection,
                &page_id,
                &session,
                vec![permissions::Permission::ModifyContent],
                csrf,
            )
            .await?;
            
            let mut _path = PathBuf::from("data");
            _path.push(page_id);
            _path.push(path);
            let path: PathBuf = check_path_traversal_attack(_path)?;

            println!("{path:?}");
            if path.is_dir() {
                fs::remove_dir_all(path.clone()).await.map_err(|_| Status::InternalServerError)?
            }
            else if path.is_file(){
                fs::remove_file(path.clone()).await.map_err(|_| Status::InternalServerError)?
            }
            else{
                return Err(Status::BadRequest);
            }
            return Ok(format!("{path:?}"));
        }
        Err(Status::BadRequest)
    }()
    .await
    .map(|path| {
        Template::render(
            "editor/components/notification",
            Notification {
                title: "Success!".into(),
                message: format!("{path} deleted."),
                ..Default::default()
            },
        )
    })
    .map_err(|e| {
        println!("{e}");
        Template::render(
            "editor/components/notification",
            Notification {
                title: "An error has occurred!".into(),
                message: format!("Error code: {}", e.code),
                ..Default::default()
            },
        )
    })
}


///Route that deletes a content from a path in the specified path and returns a Notification Component
#[put("/editor/rename", data = "<data>")]
pub async fn rename(
    mut connection: Connection<crate::DATABASE>,
    session: Session<'_, SessionToken>,
    data: Form<HashMap<String, String>>,

    csrf: &State<CSRF>,
) -> Result<Template, Template>{
    async move || -> Result<String, Status> {
        if let (Some(page_id), Some(path), Some(new_name)) = (
            data.get("page_id"),
            data.get("path"),
            data.get("new_name"),
        ) {
            let _ = permissions::get_page_if_allowed(
                &mut connection,
                &page_id,
                &session,
                vec![permissions::Permission::ModifyContent],
                csrf
            )
            .await?;
            
            let mut _path = PathBuf::from("data");
            _path.push(page_id);
            _path.push(path);
            let mut _path_to: PathBuf = _path.parent().unwrap().to_path_buf();
            _path_to.push(new_name);
            let path: PathBuf = check_path_traversal_attack(_path)?;
            let path_to: PathBuf = check_path_traversal_attack(_path_to)?;

            println!("{path:?} | {path_to:?}");

            fs::rename(path.clone(), path_to.clone()).await.map_err(|_| Status::InternalServerError)?;

            return Ok(format!("{path_to:?}"));
        }
        Err(Status::BadRequest)
    }()
    .await
    .map(|path| {
        Template::render(
            "editor/components/notification",
            Notification {
                title: "Success!".into(),
                message: format!("{path} renamed."),
                ..Default::default()
            },
        )
    })
    .map_err(|e| {
        println!("{e}");
        Template::render(
            "editor/components/notification",
            Notification {
                title: "An error has occurred!".into(),
                message: format!("Error code: {}", e.code),
                ..Default::default()
            },
        )
    })
}
