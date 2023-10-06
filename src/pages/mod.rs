use std::path::PathBuf;

use rocket::{
    fs::NamedFile,
    get,
    http::Status,
    response::{content::RawHtml, status::NotFound},
    tokio::fs,
};
use rocket_db_pools::Connection;
use serde::Serialize;

pub mod files;
pub mod page;

#[derive(sqlx::Type)]
pub enum Permission {
    Owner, //Can delete page and all permissions bellow.
    Admin, //Can create and modify articles and the permission bellow.
    Mod,   //Can delete comments.
    PrivateViewer, //This exists only if the page is set to Private.
           //only those who are invited and have the necessary permissions can access the page.
}

#[derive(sqlx::Type, Serialize)]
pub enum PageStatus {
    Public,   //Anyone can access it from anywhere. It also shows up on search websites.
    LinkOnly, //Anyone with a link can acess it. It does not show up on search websites.
    Private,  //Only a user that is logged into the website
}

