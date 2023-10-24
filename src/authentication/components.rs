use crate::configuration;
use rocket::http::Status;
use rocket::{get, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use rocket_recaptcha_v3::ReCaptcha;
use super::SessionManager;
use super::AuthParameters;

///The first screen that appears containing the login and the register forms.
#[get("/auth/components/login_register_forms")]
pub async fn login_register_forms(
    recaptcha: &State<ReCaptcha>,
    session: rocket_session_store::Session<'_, String>,
    _connection: Connection<crate::DATABASE>,
    config: &State<configuration::SharkNoteConfig>,
) -> (Status, Template){
    let template_args = AuthParameters::new(config, recaptcha);

    if let SessionManager::LoggedIn { user_id: _ } = SessionManager::get(&session).await {
        //ASK IF THE USER WANT TO LOGOUT!
        return (Status::Ok, Template::render("auth/components/already-logged-in",template_args));
    }

    return (Status::Ok, Template::render("auth/components/login-register-forms", template_args));
}
