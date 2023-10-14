use crate::configuration;
use rocket::{get, response::Redirect, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::context;
use rocket_dyn_templates::Template;
use rocket_recaptcha_v3::ReCaptcha;
use rocket_recaptcha_v3::ReCaptchaError;
use rocket_recaptcha_v3::ReCaptchaToken;
use rocket_session_store::SessionError;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub mod email;
pub mod login;
pub mod logout;
pub mod password_reset;
pub mod register;

#[derive(Serialize)]
struct FinalButton {
    href: String,
    text: String,
}

#[derive(Serialize)]
struct AuthParameters {
    mode: Option<String>,
    recaptcha: Option<String>,
    alert_level: Option<String>,
    message: Option<String>,
    final_button: Option<FinalButton>,
}
impl Default for AuthParameters {
    fn default() -> Self {
        Self {
            recaptcha: None,
            alert_level: None,
            message: None,
            mode: None,
            final_button: None,
        }
    }
}

async fn check_recaptcha(
    recaptcha: &State<ReCaptcha>,
    token: ReCaptchaToken,
) -> Result<(), Template> {
    let errors: ReCaptchaError = match recaptcha.verify(&token, None).await {
        Ok(v) => {
            if v.score > 0.7 {
                return Ok(());
            } else {
                ReCaptchaError::InternalError("You are probably a robot. If not, try again!".into())
            }
        }
        Err(e) => e,
    };
    Err(Template::render(
        "auth-panel",
        AuthParameters {
            recaptcha: recaptcha.get_html_key_as_str().map(|a| a.to_string()),
            alert_level: Some("error".into()),
            message: Some(errors.to_string()),
            ..Default::default()
        },
    ))
}

#[derive(Serialize, Deserialize)]
pub enum SessionCookie {
    LoggedIn { user_id: String },
    AwaitingConfirmation { user_id: String },
    Guest,
}

impl SessionCookie {
    pub async fn get(session: &rocket_session_store::Session<'_, String>) -> Self {
        let json = session.get().await.unwrap();
        if json.is_some() {
            let cookie: SessionCookie = serde_json::from_str(&json.unwrap()).unwrap();
            return cookie;
        }
        SessionCookie::set(&session, Self::Guest).await.unwrap();
        Self::Guest
    }
    pub async fn set(
        session: &rocket_session_store::Session<'_, String>,
        cookie: Self,
    ) -> Result<(), SessionError> {
        session.set(json!(cookie).to_string()).await
    }
    /*
    pub fn to_string(self: &Self) -> String{
        match self {
            SessionCookie::LoggedIn { user_id } => String::from("logged"),
            SessionCookie::Guest => todo!(),
        }
    }
    */
}

#[get("/auth")]
pub async fn page(
    recaptcha: &State<ReCaptcha>,
    session: rocket_session_store::Session<'_, String>,
    _connection: Connection<crate::DATABASE>,
    config: &State<configuration::SharkNoteConfig>,
) -> Result<Template, Redirect> {
    let mut template_args = AuthParameters::default();

    match SessionCookie::get(&session).await {
        SessionCookie::LoggedIn { user_id: _ } => {
            return Ok(Template::render(
                "auth-base",
                context!(
                    final_button: FinalButton{
                        href:"/auth".into(),
                        text:"Log out".into()
                    },
                    alert_level: "error",
                    message: "You are already logged in."
                ),
            ));
            //ASK IF THE USER WANT TO LOGOUT!
        }

        _ => {}
    }

    if config.auth.recaptcha {
        template_args.recaptcha = recaptcha.get_html_key_as_str().map(|a| a.to_string());
    }
    return Ok(Template::render("auth-panel", template_args));
}
