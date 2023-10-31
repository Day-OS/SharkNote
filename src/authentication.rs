use std::collections::HashMap;

use crate::configuration;
use crate::configuration::Messages;
use log::error;
use rocket::http::Status;
use rocket::{get, State};
use rocket_csrf_token::CsrfToken;
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use rocket_recaptcha_v3::ReCaptcha;
use rocket_recaptcha_v3::ReCaptchaError;
use rocket_recaptcha_v3::ReCaptchaToken;
use serde::{Deserialize, Serialize};

pub mod main_forms;
pub mod email;
pub mod login;
pub mod logout;
pub mod password_reset;
pub mod register;

pub fn check_csrf_from_hashmap(hashmap: &HashMap<String, String>, csrf: &CsrfToken) -> Result<(), Status>{
    if let Some(token) = hashmap.get("csrf_token"){
        return  check_csrf(token, csrf)
    }
    else {
        return Err(Status::BadRequest)
    }
   
}
pub fn check_csrf(token: &String, csrf: &CsrfToken) -> Result<(), Status>{
    csrf.verify(&token).map_err(|_|{Status::Forbidden})
}

#[derive(Serialize, Deserialize, Clone)]
pub enum SessionToken {
    LoggedIn { user_id: String },
    AwaitingConfirmation { user_id: String},
    Guest,
}

impl SessionToken {
    ///Gets the current sesstion Token or generate a new one in case it does not exist
    pub async fn init(session: &rocket_session_store::Session<'_, Self>) -> Self {
        match session.get().await.unwrap() {
            Some(s) => {return s}
            None => {
                let s = Self::Guest;
                _ = session.set(s).await;
                //using get instead of clone make sure it will crash if the session setting goes wrong
                session.get().await.unwrap().unwrap()
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
    fn new(config: &State<configuration::SharkNoteConfig>,) -> Self {
        Self {
            recaptcha_key: None,
            messages: config.messages.clone(),
            alert: None,
            final_button: None,
            csrf_token:None
        }
    }
    fn add_recaptcha(mut self, recaptcha: &State<ReCaptcha>)-> Self{
        self.recaptcha_key = recaptcha.get_html_key_as_str().map(|a| a.to_string());
        self
    }
    fn add_csrf_token(mut self, csrf: &CsrfToken) -> Self{
        self.csrf_token = Some(csrf.authenticity_token());
        self
    }
    fn map<F>(self, f:F)-> Self where F:Fn(Self)->Self{
        f(self)
    }
}

/// This code checks the recaptcha token if the config indicates that recaptcha is enabled.
/// This code is called by the login and register endpoints, and it returns an error if the recaptcha token is not valid.
/// The recaptcha token is verified by the recaptcha service, and the verification score is checked to make sure it is above a certain threshold.
/// If the score is above the threshold, the function returns Ok(()), otherwise it returns an Unauthorized error.
/// The score threshold is 0.7.
async fn check_recaptcha_token(
    config: &State<configuration::SharkNoteConfig>,
    recaptcha: &State<ReCaptcha>,
    token: &Option<ReCaptchaToken>,
) -> Result<(), Status> {
    if config.auth.recaptcha {
        let verification = recaptcha.verify(&token.clone().unwrap(), None)
        .await
        .map_err(|e|{
            error!("{e}");
            match e {
                ReCaptchaError::InvalidInputSecret | ReCaptchaError::InvalidReCaptchaToken => Status::BadRequest,
                ReCaptchaError::TimeoutOrDuplicate | ReCaptchaError::InternalError(_) => Status::InternalServerError,
            }
        })?;
        if verification.score >= 0.7 {
            return Ok(());
        } else {
            return Err(Status::Unauthorized);
        }
    }
    return Ok(());
}

#[get("/auth")]
pub async fn base(
    recaptcha: &State<ReCaptcha>,
    _connection: Connection<crate::DATABASE>,
    config: &State<configuration::SharkNoteConfig>,
    csrf: CsrfToken,
) -> (Status, Template) {
    let template_args = AuthParameters::new(config).add_recaptcha(recaptcha).add_csrf_token(&csrf);
    return (Status::Ok ,Template::render("auth/auth", template_args));
}