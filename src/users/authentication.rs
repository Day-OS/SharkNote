use std::{
    collections::HashMap,
    net::SocketAddr,
};

use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};
use log::error;
use rand::Rng;
use rocket::{
    form::Form, 
    post,
    response::Redirect,
    uri, FromForm, State, get,
};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use rocket_dyn_templates::context;
use rocket_recaptcha_v3::ReCaptchaToken;
use rocket_recaptcha_v3::ReCaptcha;
use rocket_session_store::SessionError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use strfmt::strfmt;

use crate::{configuration, users::user::User};

use super::user;
use super::user::UserAccountStatus;

fn send_message(alert_level: String, message: String) -> Template {
    let mut hashmap = HashMap::new();
    hashmap.insert("alert_level", alert_level);
    hashmap.insert("message", message);
    Template::render("auth-message", hashmap)
}

async fn check_recaptcha(recaptcha: &State<ReCaptcha>, token: ReCaptchaToken) -> Result<(), String> {
    match recaptcha.verify(&token, None).await {
        Ok(v) => {
            if v.score > 0.7 {
                Ok(())
            }
            else{
                Err("You are probably a robot. If not, try again!".into())
            }
        }
        Err(e) => {Err(e.to_string())}
    }
}

#[derive(Serialize, Deserialize)]
pub enum SessionCookie {
    LoggedIn { user_id: String },
    AwaitingConfirmation { user_id: String },
    Guest,
    GuestError { auth_error: String },
}

impl SessionCookie {
    pub async fn get(session: rocket_session_store::Session<'_, String>) -> Self {
        println!("haiii, {:?}", session.get().await.unwrap() );
        let json = session.get().await.unwrap();
        if json.is_some() {
            let cookie: SessionCookie = serde_json::from_str(&json.unwrap()).unwrap();
            return cookie;
        }
        SessionCookie::set(session, Self::Guest).await.unwrap();
        Self::Guest
        
    }
    pub async fn set(
        session: rocket_session_store::Session<'_, String>,
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

#[derive(FromForm)]
pub struct UserLoginForm {
    pub user_id: String,
    pub password: String,
}

#[derive(FromForm, Debug)]
pub struct UserRegistrationForm {
    user_id: String,
    password: String,
    email: String,
    recaptcha_token: Option<ReCaptchaToken>,
}

#[derive(FromForm, Debug)]
pub struct ConfirmationForm {
    code: String,
}

#[post("/auth/login", data = "<form>")]
pub async fn login(
    session: rocket_session_store::Session<'_, String>,
    form: Form<UserLoginForm>,
    mut connection: Connection<crate::DATABASE>,
) -> Redirect {
    match User::check_login_credentials(
        &mut *connection,
        form.user_id.clone(),
        form.password.clone(),
    )
    .await
    {
        Ok(right_credentials) => {
            if right_credentials {
                json!(SessionCookie::LoggedIn {
                    user_id: form.user_id.clone()
                });
                _ = session.set(form.user_id.clone()).await;
                return Redirect::to(uri!("/editor"));
            } else {
                return Redirect::to(format!("/auth?error={}", "401"));
            }
        }
        Err(_) => {
            return Redirect::to(format!("/auth?error={}", "500"));
        }
    };
}

#[post("/auth/register", data = "<form>")]
pub async fn register(
    recaptcha: &State<ReCaptcha>,
    remote_addr: SocketAddr,
    session: rocket_session_store::Session<'_, String>,
    form: Form<UserRegistrationForm>,
    config: &State<configuration::SharkNoteConfig>,
    mut connection: Connection<crate::DATABASE>,
) -> Result<Template, Redirect> {
    if let Err(error) =  check_recaptcha(recaptcha, form.recaptcha_token.clone().unwrap()).await{
        SessionCookie::set(session,
            SessionCookie::GuestError {
                auth_error: error,
            }).await.unwrap();
        return Err(Redirect::moved("/auth/"));
    }

    if let Err(e) = User::new(
        &mut *connection,
        form.user_id.clone(),
        form.password.clone(),
        form.email.clone(),
        user::UserAccountStatus::Normal,
    )
    .await{
        error!("{e}");
        return Ok(send_message(
            "error".into(),
            config.messages.account_creation_error.clone(),
        ))
    }

    if let Some(smtp) = &config.smtp {
        let code: u32 = rand::thread_rng().gen_range(0..999999);
        user::User::get(&mut connection, form.user_id.clone()).await.unwrap()
            .set_status(&mut connection, user::UserAccountStatus::RegistrationPending { code: code }).await.unwrap();
        let mut vars = HashMap::new();
        vars.insert(
            "display_program_name".to_string(),
            config.messages.display_program_name.clone(),
        );
        vars.insert("user_id".to_string(), form.user_id.clone());
        vars.insert("confirmation_code".to_string(), code.to_string());

        let email = Message::builder()
            .from(format!("<{}>", smtp.smtp_username.clone()).parse().unwrap())
            .to(format!("<{}>", form.email.clone()).parse().unwrap())
            .subject(strfmt(&config.messages.email_registration_title, &vars).unwrap())
            .header(ContentType::TEXT_PLAIN)
            .body(strfmt(&config.messages.email_registration_text, &vars).unwrap())
            .unwrap();

        let creds = Credentials::new(smtp.smtp_username.clone(), smtp.smtp_password.clone());

        // Open a remote connection to gmail
        let mailer = SmtpTransport::relay(&smtp.smtp_relay)
            .unwrap()
            .credentials(creds)
            .build();

        // Send the email
        if let Err(e) = mailer.send(&email) {
            error!("{e}");
            return Ok(send_message(
                "error".into(),
                config.messages.account_email_send_error.clone(),
            ))
        }
        SessionCookie::set(session, SessionCookie::AwaitingConfirmation { user_id: form.user_id.clone() }).await.unwrap();
        return Ok(Template::render("auth-confirmation", context! {
            mode: "registration",
            errormessage: "Wrong code"
        }));
    }
    return Ok(send_message(
        "success".into(),
        config.messages.account_creation_success.clone(),
    ))
}

#[post("/auth/confirmation", data = "<form>")]
pub async fn confirmation(
    session: rocket_session_store::Session<'_, String>,
    form: Form<ConfirmationForm>,
    config: &State<configuration::SharkNoteConfig>,
    mut connection: Connection<crate::DATABASE>,
) -> Result<Template, Redirect> {
    let session = SessionCookie::get(session).await;

    if let SessionCookie::AwaitingConfirmation { user_id } = session {
        let user: User = user::User::get(&mut connection, user_id).await.unwrap();
        if let UserAccountStatus::RegistrationPending { code } = user::UserAccountStatus::from_json(user.account_status.clone()){
            if form.code == code.to_string(){
                user.set_status(&mut connection, UserAccountStatus::Normal).await.unwrap();
                return Ok(send_message(
                    "success".into(),
                    config.messages.account_creation_success.clone(),
                ))
            }
        }
        return Ok(Template::render("auth-confirmation", context! {
            mode: "registration"
        }))
    } 
    return Err(Redirect::to("/auth"));
}

#[get("/auth")]
pub async fn auth_page(
    recaptcha: &State<ReCaptcha>,
    session: rocket_session_store::Session<'_, String>,
    connection: Connection<crate::DATABASE>,
    config: &State<configuration::SharkNoteConfig>,
    headers: crate::RequestHeaders<'_>,
) -> Result<Template, Redirect> {
    let mut template_args = HashMap::new();

    //Manages errors and also returns the user back a page if he is already LoggedIn
    match SessionCookie::get(session).await {
        SessionCookie::GuestError { auth_error } => {
            template_args.insert("errormessage", auth_error);
        }
        SessionCookie::LoggedIn { user_id: _ } => {
            return Err(Redirect::to(
                headers.0.get("Referer").next().unwrap().to_string(),
            ))
        }

        _ => {}
    }

    if config.auth.recaptcha {
        template_args.insert(
            "recaptcha",
            recaptcha
                .get_html_key_as_str()
                .map(|a| a.to_string())
                .unwrap(),
        );
    }
    return Ok(Template::render("auth", template_args));
}
