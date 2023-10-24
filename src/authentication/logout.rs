use rocket::{get, response::Redirect, State, http::Status};
use rocket_dyn_templates::Template;
use rocket_recaptcha_v3::ReCaptcha;

use crate::configuration;

use super::{SessionManager, AuthParameters};

#[get("/auth/logout")]
pub async fn logout(session: rocket_session_store::Session<'_, String>, 
config: &State<configuration::SharkNoteConfig>,
recaptcha: &State<ReCaptcha>,
) -> (Status, Template) {
    let template_args = AuthParameters::new(config, recaptcha);
    let _ = SessionManager::set(&session, SessionManager::Guest).await;
    return (Status::Ok, Template::render("auth/components/login-register-forms", template_args));
}
