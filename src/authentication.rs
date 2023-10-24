use crate::configuration;
use crate::configuration::Messages;
use crate::configuration::SharkNoteConfig;
use log::info;
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

pub mod components;
pub mod email;
pub mod login;
pub mod logout;
pub mod password_reset;
pub mod register;



#[derive(Serialize, Deserialize, Clone)]
pub enum SessionManager {
    LoggedIn { user_id: String },
    AwaitingConfirmation { user_id: String },
    Guest ,
}

impl SessionManager {
    pub async fn get(session: &rocket_session_store::Session<'_, String>) -> Self {
        let json = session.get().await.unwrap();
        if json.is_some() {
            let cookie: SessionManager = serde_json::from_str(&json.unwrap()).unwrap();
            return cookie;
        }
        SessionManager::set(&session, Self::Guest).await.unwrap();
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
            SessionManager::LoggedIn { user_id } => String::from("logged"),
            SessionManager::Guest => todo!(),
        }
    }
    */
}


#[derive(Serialize, Clone)]
enum AlertLevel {
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "success")]
    Success,
}

#[derive(Serialize, Clone)]
struct Alert {
    alert_level: AlertLevel,
    message: String,
}

#[derive(Serialize, Clone)]
struct FinalButton {
    href: String,
    text: String,
}

#[derive(Serialize, Clone)]
struct AuthParameters {
    recaptcha_key: Option<String>,
    alert: Option<Alert>,
    final_button: Option<FinalButton>,
    messages: Messages,
}

impl AuthParameters {
    fn new(config: &State<configuration::SharkNoteConfig>, recaptcha: &State<ReCaptcha>) -> Self {
        Self {
            recaptcha_key: recaptcha.get_html_key_as_str().map(|a| a.to_string()),
            messages: config.messages.clone(),
            alert: None,
            final_button: None,
        }
    }
    fn map<F>(self, f:F)-> Self where F:Fn(Self)->Self{
        f(self)
    }
}

///Always returns Ok if recaptcha is disabled.
async fn check_recaptcha_token(
    config: &State<configuration::SharkNoteConfig>,
    recaptcha: &State<ReCaptcha>,
    token: Option<ReCaptchaToken>,
) -> Result<(), Template> {
    if config.auth.recaptcha {
        async || -> Result<(), ReCaptchaError> {
            let verification = recaptcha.verify(&token.unwrap(), None).await?;
            if verification.score >= 0.7 {
                return Ok(());
            } else {
                Err(ReCaptchaError::InternalError(
                    "You are probably a robot. If not, try again!".into(),
                ))
            }
        }()
        .await
        .map_err(|e| {
            Template::render(
                "auth/base",
                AuthParameters::new(config, recaptcha).map(|mut auth|{
                    auth.alert = Some(Alert {
                        alert_level: AlertLevel::Error,
                        message: e.to_string(),
                    });
                    auth
                }),
            )
        })
    } else {
        Ok(())
    }
}

#[get("/auth")]
pub async fn base(
    recaptcha: &State<ReCaptcha>,
    _connection: Connection<crate::DATABASE>,
    config: &State<configuration::SharkNoteConfig>,
) -> Result<Template, Redirect> {
    let mut template_args = AuthParameters::new(config, recaptcha);
    return Ok(Template::render("auth/auth", template_args));
}
