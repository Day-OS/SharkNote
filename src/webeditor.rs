use rocket::{response::{Redirect}, form::Form, State, post, get, FromForm, uri, figment::util};
use rocket_dyn_templates::{Template, context};
use rocket_session_store::{Session, SessionError};
use rusqlite::Connection;
use serde::Serialize;
use crate::{user, pages::{self, Page, PAGE_DIR}, utils::{Directory, self, get_directory_file_names}};
use crate::utils::DBErrors::*;
use crate::language_file::get_file;


#[derive(FromForm)]
pub struct createPage {
    pagename: String,
}

#[derive(FromForm)]
pub struct PageOwnerForm {
    user_id: String,
    password: String,
}

#[derive(Serialize)]
struct  EditorParameters<>{
    is_page_from_owner:bool,
    user_id: String,  
    page_name: String,
    owned_pages: Vec<Page>,
    files: Directory,//Vec<String>
}
impl Default for EditorParameters {
    fn default() -> EditorParameters {
        EditorParameters {
            is_page_from_owner: false, 
            user_id: "".into(), 
            page_name: "".into(), 
            owned_pages:vec![], 
            files: Directory{name: "?".into(),directories:vec![],files:vec![]}
        }
    }
}


#[post("/login", data="<form>")]
pub async fn post_editor(session: Session<'_, String>, form : Form<PageOwnerForm>, db: &State<crate::DbConn>) -> Redirect{
    match user::check_login_credentials(&form.user_id, &form.password, db) {
        Ok(response)=>{
            if response {
                session.set(form.user_id.clone()).await;
                println!("User {} logged in successufully!", form.user_id);
                Redirect::to(uri!("/editor"))
            }
            else{
                return Redirect::to(format!("/editor?error={}","wrongpassword"))
            }
            
        }

        Err(error)=>{
            match error {
                CantDeleteUser(_, _) | ProgramError(_) | GenericNotFound => {
                    return Redirect::to(format!("/editor?error={}","wrongpassword"))
                },

                SqliteError(e) => {println!("erroooo: {}", e);}
            }
            Redirect::to(format!("/editor?error={}","?"))
        }
    }
}

#[post("/create_page", data="<form>")]
pub async fn create_page(session: Session<'_, String>, form : Form<createPage>, db: &State<crate::DbConn>) -> Result<Redirect, SessionError>{
    let name = match session.get().await? {
        Some(name) => name,
        None => return Err(SessionError)
    };
    let database = match db.lock() {
        Ok(database) => database,
        Err(_)=> return Err(SessionError)
    };
    pages::create_page(&database, form.pagename.clone());
    pages::set_page_admin(&database, &form.pagename.clone(), &name);
    return Ok(Redirect::to(format!("/editor?page={}","wrongpassword")))
}



#[get("/editor?<error>&<page>")]
pub async fn editor(session: Session<'_, String>, db: &State<crate::DbConn> , error: Option<&str>, page: Option<&str>) -> Result<Template, SessionError>{
    let language = get_file();
    
    
    let user_id: Option<String> = match session.get().await? {
        Some(user_id) => Some(user_id),
        None =>{None}
    };

    let database = match db.lock() {
        Ok(database) => database,
        Err(_)=> return Err(SessionError)
    };

    fn manage_editor_template_parameters(database: &Connection, editor_params: &mut EditorParameters){

        
        match pages::get_owned_pages(&database, &editor_params.user_id) {
            Ok(vec)=>{
                editor_params.owned_pages = vec;
            },
            Err(e)=>{match e {
                SqliteError(e)=>{println!("{}", e)}
                _=>{}
            }}
        }
        
        //TO-DO: SEND EVERY OWNED PAGE TO HTML TEMPLATE
        // /editor_params.owned_pages;

        if !pages::is_page_admin(&database, &editor_params.page_name, &editor_params.user_id){return;}
        editor_params.is_page_from_owner = true;

        let mut page_dir = utils::get_program_files_location();
        page_dir.push_str(PAGE_DIR);
        page_dir.push_str(&editor_params.page_name);
        editor_params.files = get_directory_file_names(page_dir,None);//pages::get_pages_files_list(&editor_params.page_name);
        //TO-DO: SEND EVERY FILE INSIDE PAGE TO HTML TEMPLATE
        //editor_params.files;
    
    }

    match user_id {
        Some(user_id) =>{
            let mut editor_params = EditorParameters{user_id:user_id,..Default::default()};
            match page {None=>{} Some(page_name)=>{
                editor_params.page_name = page_name.into();
            }}
            manage_editor_template_parameters(&database, &mut editor_params);
            return Ok(Template::render("editor", context!{editor_params:editor_params}))
        } 
        None =>{}
    }
    

    match error {
        Some(text) => {
            let error = match text {
                "wrongpassword" => language.wrong_password,
                "internalerror" => language.program_error,
                _ => "undefined error".to_string()
            };
            return Ok(Template::render("error", context!{errormessage:error}));
        }None => {},
    };
    return Ok(Template::render("login", context!{}))
}