use rocket::{
    figment::util,
    form::Form,
    get,
    http::{hyper::Request, Status},
    post,
    request::FromRequest,
    response::Redirect,
    uri, FromForm, Response, State,
};
use rocket_db_pools::Connection;
use rocket_dyn_templates::{context, Template};
use rocket_session_store::{Session, SessionError};
use serde::Serialize;

use crate::{pages::page::Page, users::user::User};

#[derive(FromForm)]
pub struct createPage {
    pagename: String,
}

#[derive(Serialize)]
struct EditorParameters<'a> {
    user_id: String,
    page_id: Option<&'a str>,
}

#[get("/editor?<page_id>")]
pub async fn editor(
    session: Session<'_, String>,
    connection: Connection<crate::DATABASE>,
    page_id: Option<&str>,
) -> Result<Template, Redirect> {
    let user_id: String = match session.get().await.unwrap() {
        Some(id) => id,
        None => return Err(Redirect::to("/login")), // None=>{return Ok(Template::render("redirect", context! {}))}
    };

    let editor_params = EditorParameters {
        user_id: user_id,
        page_id: page_id,
    };
    return Ok(Template::render(
        "editor",
        context! {editor_params:editor_params},
    ));
}
/*

#[post("/editor/create_page", data = "<form>")]
pub async fn create_page(
    session: Session<'_, String>,
    form: Form<createPage>,
    connection: Connection<crate::DATABASE>,
) -> Result<Redirect, SessionError> {
    let name = match session.get().await? {
        Some(name) => name,
        None => return Err(SessionError),
    };
    let database = match db.lock() {
        Ok(database) => database,
        Err(_) => return Err(SessionError),
    };
    pages::create_page(&database, form.pagename.clone());
    pages::set_page_admin(&database, &form.pagename.clone(), &name);
    return Ok(Redirect::to(format!("/editor?page={}", form.pagename)));
}
 */
