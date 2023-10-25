use rocket::{get, State, http::Status};
use rocket_dyn_templates::Template;
use rocket_recaptcha_v3::ReCaptcha;
use crate::authentication::CSRF;
use crate::configuration;

use super::{SessionToken, AuthParameters};

#[get("/auth/logout")]
pub async fn logout(session: rocket_session_store::Session<'_, SessionToken>, 
config: &State<configuration::SharkNoteConfig>,
recaptcha: &State<ReCaptcha>,
csrf: &State<CSRF>,
) -> (Status, Template) {
    let template_args = AuthParameters::new(config, recaptcha);
    let _ = session.set(SessionToken::Guest { csrf_token: csrf.new_token()}).await;
    return (Status::Ok, Template::render("auth/components/login-register-forms", template_args));
}
