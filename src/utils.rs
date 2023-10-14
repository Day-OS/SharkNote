use std::path::{PathBuf, Component};

use rocket::http::Status;


pub fn check_path_traversal_attack(path: &PathBuf) -> Result<&PathBuf, Status>{
    if path.components().into_iter().any(|x| x == Component::ParentDir) {
        return Err(Status::Unauthorized);
    }
    Ok(path)
}