use crate::configuration;
use crate::configuration::Messages;
use crate::configuration::SharkNoteConfig;
use csrf::AesGcmCsrfProtection;
use csrf::CsrfProtection;
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

pub mod main_forms;
pub mod email;
pub mod login;
pub mod logout;
pub mod password_reset;
pub mod register;


pub struct CSRF(pub AesGcmCsrfProtection);

impl CSRF{
    fn new_token(&self) -> String{
        self.0.generate_token_pair(None, 300).unwrap().0.b64_string()
    }
    fn a(&self, token: String){

        self.0.parse_token(token.as_bytes()).unwrap()
    }
}


#[derive(Serialize, Deserialize, Clone)]
pub enum SessionToken {
    LoggedIn { user_id: String, csrf_token: String },
    AwaitingConfirmation { user_id: String , csrf_token: String},
    Guest {csrf_token: String},
}

impl SessionToken {
    pub async fn init(session: &rocket_session_store::Session<'_, Self>, csrf: &CSRF) -> Self {
        match session.get().await.unwrap() {
            Some(mut s) => {
                match &mut s {
                    SessionToken::LoggedIn { user_id: _, csrf_token } |
                    SessionToken::AwaitingConfirmation { user_id: _, csrf_token } |
                    SessionToken::Guest { csrf_token } => {
                        if csrf.0.parse_token(csrf_token.as_bytes()).is_err(){
                            *csrf_token = csrf.new_token();
                        }
                    },
                }
                return s
            }
            None => {
                let s = Self::Guest{csrf_token: csrf.new_token()};
                session.set(s.clone()).await;
                s
            },
        }
        
    }
    //csrf: &State<CSRF>,
    /*
    pub fn to_string(self: &Self) -> String{
        match self {
            Session::LoggedIn { user_id } => String::from("logged"),
            Session::Guest => todo!(),
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
    csrf_token: Option<String>,
}

impl AuthParameters {
    fn new(config: &State<configuration::SharkNoteConfig>, recaptcha: &State<ReCaptcha>,) -> Self {
        Self {
            recaptcha_key: recaptcha.get_html_key_as_str().map(|a| a.to_string()),
            messages: config.messages.clone(),
            alert: None,
            final_button: None,
            csrf_token:None
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
    csrf: &State<CSRF>,
) -> Result<Template, Redirect> {
    let mut template_args = AuthParameters::new(config, recaptcha).map(|a| a.csrf_token = Some(()));
    return Ok(Template::render("auth/auth", template_args));
}
