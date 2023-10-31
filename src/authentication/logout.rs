use std::collections::HashMap;

use rocket::{State, http::Status, form::Form, post};
use rocket_csrf_token::CsrfToken;
use rocket_dyn_templates::Template;
use rocket_recaptcha_v3::ReCaptcha;
use crate::configuration;

use super::{SessionToken, AuthParameters};

#[post("/auth/logout", data="<data>")]
pub async fn component(session: rocket_session_store::Session<'_, SessionToken>, 
config: &State<configuration::SharkNoteConfig>,
data: Form<HashMap<String, String>>,
recaptcha: &State<ReCaptcha>,
csrf: CsrfToken,
) -> (Status, Option<Template>) {
    if let Err(status) = super::check_csrf_from_hashmap(&data, &csrf){
        return (status, None)
    };

    let template_args = AuthParameters::new(config).add_recaptcha(recaptcha);
    let _ = session.set(SessionToken::Guest).await;
    return (Status::Ok, Some(Template::render("auth/components/main-forms", template_args)));
}
