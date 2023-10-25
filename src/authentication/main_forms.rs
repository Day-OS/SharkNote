use std::collections::HashMap;

use crate::configuration;
use rocket::form::Form;
use rocket::http::Status;
use rocket::{get, State, post};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use rocket_recaptcha_v3::ReCaptcha;
use super::{SessionToken, CSRF};
use super::AuthParameters;

///The first screen that appears containing the login and the register forms.
#[post("/auth/components/main_forms", data="<data>")]
pub async fn login_register_forms(
    recaptcha: &State<ReCaptcha>,
    session: rocket_session_store::Session<'_, SessionToken>,
    data: Form<HashMap<String, String>>,
    csrf: &State<CSRF>,
    _connection: Connection<crate::DATABASE>,
    config: &State<configuration::SharkNoteConfig>,
) -> (Status, Template){
    let template_args = AuthParameters::new(config, recaptcha);
    ;
    if let SessionToken::LoggedIn { user_id: _, csrf_token } = SessionToken::init(&session, csrf).await {
        //ASK IF THE USER WANT TO LOGOUT!
        return (Status::Ok, Template::render("auth/components/already-logged-in",template_args));
    }

    return (Status::Ok, Template::render("auth/components/main-forms", template_args));
}
