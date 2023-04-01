use html_editor::{operation::{Editable, Selector, Htmlifiable}, Node};
use rocket::{response::{content::{RawHtml, self}, Redirect}, serde::__private::doc, http::{Cookie, CookieJar}, form::Form, State, post, get, FromForm, uri, error};
use rocket_dyn_templates::{Template, context};
use rocket_session_store::{Session, SessionError};
use std::{fs::File, path};
use crate::{page_owner, db_utils};
use crate::db_utils::DBErrors::*;
use crate::language_file::get_file;

#[derive(FromForm)]
pub struct PageOwnerForm {
    page_owner_id: String,
    password: String,
}


#[post("/login", data="<form>")]
pub async fn post_editor(session: Session<'_, String>, form : Form<PageOwnerForm>, db: &State<crate::DbConn>) -> Redirect{
    match page_owner::check_login_credentials(&form.page_owner_id, &form.password, db) {
        Ok(response)=>{
            if response {
                session.set(form.page_owner_id.clone()).await;
                println!("User {} logged in successufully!", form.page_owner_id);
                Redirect::to(uri!("/p/editor"))
            }
            else{
                return Redirect::to(format!("/p/editor?error={}","wrongpassword"))
            }
            
        }

        Err(error)=>{
            match error {
                CantDeleteUser(_, _) | ProgramError(_) | GenericNotFound => {
                    return Redirect::to(format!("/p/editor?error={}","wrongpassword"))
                },

                SqliteError(e) => {println!("erroooo: {}", e);}
            }
            Redirect::to(format!("/p/editor?error={}","?"))
        }
    }
}

#[get("/editor?<error>")]
pub async fn editor(session: Session<'_, String>, error: Option<&str>) -> Result<Template, SessionError>{
    let language = get_file();
    //Checks if there's a section, if there is it renders the editor template.
    match session.get().await? {
        Some(name) =>{ return Ok(Template::render("editor", context!{}))} 
        None =>{}
    };

    match error {
        Some(text) => {
            println!("wooooo {}", text);
            match text {
                "wrongpassword" => return Ok(Template::render("error", context!{errormessage:language.wrong_password})),
                "internalerror" => return Ok(Template::render("error", context!{errormessage:language.program_error})),
                _ =>{}
            }
        },
        None => {println!("???");},
    };
    return Ok(Template::render("login", context!{}))
}