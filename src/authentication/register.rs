use super::check_recaptcha_token;
use super::AuthParameters;
use super::email::Code;
use super::email::send_register_code;
use crate::users::invite;
use crate::users::UserAccountStatus;
use crate::{configuration, users::User};
use log::error;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{form::Form, post, FromForm, State};
use rocket_csrf_token::CsrfToken;
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use rocket_recaptcha_v3::ReCaptcha;
use rocket_recaptcha_v3::ReCaptchaToken;
use serde::{Deserialize, Serialize};

use super::SessionToken;




#[derive(Serialize, Deserialize)]
pub(crate) struct Check {
    pub user_id: bool,
    pub email: bool,
}

///Check if the current values exist, if so, the user won't be able to register.
#[post("/auth/check", data = "<form>")]
pub async fn check(
    form: Form<RegistrationForm>,
    mut connection: Connection<crate::DATABASE>,
) -> Json<Check> {
    return Json(Check {
        user_id: User::get(&mut connection, form.user_id.clone())
            .await
            .is_ok(),
        email: User::get_from_email(&mut connection, form.email.clone())
            .await
            .is_ok(),
    });
}

#[derive(FromForm, Debug)]
pub(crate) struct ConfirmationForm {
    code: String,
    pub csrf_token: String,
    pub recaptcha_token: Option<ReCaptchaToken>,
}

#[post("/auth/register-conf", data = "<form>")]
pub async fn confirmation(
    recaptcha: &State<ReCaptcha>,
    session: rocket_session_store::Session<'_, SessionToken>,
    form: Form<ConfirmationForm>,
    config: &State<configuration::SharkNoteConfig>,
    mut connection: Connection<crate::DATABASE>,
    csrf: CsrfToken,
) -> (Status, Template) {

    let parameters = AuthParameters::new(config).add_recaptcha(recaptcha);
    let session = SessionToken::init(&session).await;
    
    let code = async || -> Status{
        if let Err(st) = super::check_csrf(&form.csrf_token, &csrf) {
            return st;
        };
        if let Err(st) = check_recaptcha_token(config, recaptcha, &form.recaptcha_token).await {
            return st;
        };        
        if let SessionToken::AwaitingConfirmation { user_id } = session {
            let user: User = User::get(&mut connection, user_id).await.unwrap();
            if let Ok(code) = Code::get(&mut connection, &user).await {
                if form.code == code.code.to_string() {
                    code.delete(&mut connection).await.unwrap();
                    user.set_status(&mut connection, UserAccountStatus::Normal)
                        .await
                        .unwrap();
                    return Status::Ok;
                }
                return Status::Forbidden;
            }
            return Status::InternalServerError;
        }
        Status::InternalServerError
    }().await;
    match code {
        //User Logged in successfully and can access his account.
        Status { code: 200 } => {
            return (
                Status::Ok,
                Template::render("auth/components/login-success", parameters),
            );
        }
        //Wrong code
        Status { code: 403 } => {
            return (
                Status::Forbidden,
                Template::render("auth/components/alert-wrong-code", parameters),
            );
        }
        Status { code: 500 } | _ => {
            return (
                Status::InternalServerError,
                Template::render("auth/components/alert-internal-error", parameters),
            );
        }
    }
}


#[derive(FromForm, Debug)]
pub(crate) struct RegistrationForm {
    pub user_id: String,
    pub password: String,
    pub email: String,
    pub csrf_token: String,
    pub recaptcha_token: Option<ReCaptchaToken>,
}

#[post("/auth/register", data = "<form>")]
pub async fn post(
    mut connection: Connection<crate::DATABASE>,
    recaptcha: &State<ReCaptcha>,
    session: rocket_session_store::Session<'_, SessionToken>,
    form: Form<RegistrationForm>,
    config: &State<configuration::SharkNoteConfig>,
    csrf: CsrfToken,
) -> (Status, Template) {
    let parameters = AuthParameters::new(config);

    let code = async move || -> Status {
        if let Err(st) = super::check_csrf(&form.csrf_token, &csrf) {
            return st;
        };
        if let Err(st) = check_recaptcha_token(config, recaptcha, &form.recaptcha_token).await {
            return st;
        };        

        if config.auth.invite_only {
            if !invite::Invite::is_email_invited(&mut *connection, form.email.clone()).await {
                return Status::Unauthorized
            };
        }
        let user_result = User::new(
            &mut *connection,
            form.user_id.clone(),
            form.password.clone(),
            form.email.clone(),
            UserAccountStatus::Normal,
        )
        .await;

        if let Err(e) = user_result {
            error!("{e}");
            return e;
        }

        let user = user_result.unwrap();

        // IF SMTP Server is enabled
        if config.smtp.is_some() {
            if let Err(e) = send_register_code(&mut connection, config, &user).await {
                error!("{e} | During registration");
                return Status::InternalServerError
            }
            let _ = user.set_status(&mut *connection, UserAccountStatus::RegistrationPending).await;
            if let Err(_) = session.set(SessionToken::AwaitingConfirmation {user_id: form.user_id.clone()})
            .await{
                return Status::InternalServerError
            };
            return Status::Accepted;
        }
        

        Status::Ok
    }().await;

    match code {
        //User Logged in successfully and can access his account.
        Status { code: 200 } => {
            return (
                Status::Ok,
                Template::render("auth/components/register-success", parameters),
            );
        }
        //User Logged in, but have to insert the email code
        Status { code: 202 } => {
            return (
                Status::Accepted,
                Template::render("auth/components/register-confirmation-code", parameters),
            );
        }
        //Wrong Password
        Status { code: 403 } => {
            return (
                Status::Forbidden,
                Template::render("auth/components/alert-regex-fail", parameters),
            );
        }
        Status { code: 500 } | _ => {
            return (
                Status::InternalServerError,
                Template::render("auth/components/alert-internal-error", parameters),
            );
        }
    }

    
    
}
