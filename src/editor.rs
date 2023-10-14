use std::collections::HashMap;

use crate::{
    authentication::{SessionCookie},
    pages, users,
};
use rocket::{get, response::Redirect, FromForm};
use rocket_db_pools::Connection;
use rocket_dyn_templates::{context, Template};
use rocket_session_store::Session;
use serde::Serialize;
use users::User;
use pages::Page;
pub mod file_explorer;

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
    mut connection: Connection<crate::DATABASE>,
    page_id: Option<&str>,
) -> Result<Template, Redirect> {
    //TEMPORARY FOR TESTS
    SessionCookie::set(
        &session,
        SessionCookie::LoggedIn {
            user_id: "dayos".into(),
        },
    )
    .await;
    let returnal_error = Redirect::to("/");
    if let SessionCookie::LoggedIn { user_id } = SessionCookie::get(&session).await {
        let user = User::get(&mut *connection, user_id.clone())
            .await
            .map_err(|_| returnal_error)?;
        let pages = user.get_modifiable_pages(&mut *connection).await.ok();
        let select_page = Template::render("editor/select_page", context! {pages:&pages}); //for when the page is not specified
        if page_id.is_none() {
            return Ok(select_page);
        }
        let page = pages::Page::get(&mut *connection, page_id.unwrap().to_string()).await;
        if page.is_err() {
            return Ok(select_page);
        }

        let mut editor_params = HashMap::new();
        editor_params.insert("user_id", Some(user_id));
        editor_params.insert("page_id", page_id.map(|id| id.to_string()));
        return Ok(Template::render(
            "editor/editor",
            context! {
                page_id: page_id,
                editor_params:editor_params,
                pages:pages
            },
        ));
    } else {
        return Err(Redirect::to("/auth"));
    }
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
