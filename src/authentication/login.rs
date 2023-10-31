use log::error;

use crate::{configuration, users::User};
use rocket::http::Status;
use rocket::{form::Form, post, FromForm, State};
use rocket_csrf_token::CsrfToken;
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use rocket_recaptcha_v3::ReCaptcha;
use rocket_recaptcha_v3::ReCaptchaToken;

use super::check_recaptcha_token;
use super::email;
use super::AuthParameters;

use super::email::Code;
use super::SessionToken;

#[derive(FromForm, Debug)]
pub(crate) struct ConfirmationForm {
    code: String,
    csrf_token: String,
    recaptcha_token: Option<ReCaptchaToken>,
}

#[post("/auth/login/confirmation", data = "<form>")]
pub async fn confirmation(
    session: rocket_session_store::Session<'_, SessionToken>,
    recaptcha: &State<ReCaptcha>,
    form: Form<ConfirmationForm>,
    config: &State<configuration::SharkNoteConfig>,
    mut connection: Connection<crate::DATABASE>,
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
        if let SessionToken::AwaitingConfirmation { user_id } = SessionToken::init(&session).await {
            let user: User = User::get(&mut connection, user_id.clone()).await.unwrap();
            if let Ok(code) = Code::get(&mut connection, &user).await {
                if form.code == code.code.to_string() {
                    if let Err(e) = session
                        .set(SessionToken::LoggedIn {
                            user_id: user_id.clone(),
                        })
                        .await
                    {
                        error!("{e}");
                        return Status::InternalServerError;
                    }
                    if let Err(_) = Code::delete(code, &mut connection).await{ return Status::InternalServerError;}
                    return Status::Ok;
                }
                else{return Status::Forbidden}
            }
            return Status::InternalServerError
        }
        Status::InternalServerError
    }()
    .await;

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
pub(crate) struct LoginForm {
    user_id: String,
    password: String,
    csrf_token: String,
    recaptcha_token: Option<ReCaptchaToken>,
}

#[post("/auth/login", data = "<form>")]
pub async fn post(
    recaptcha: &State<ReCaptcha>,
    session: rocket_session_store::Session<'_, SessionToken>,
    form: Form<LoginForm>,
    config: &State<configuration::SharkNoteConfig>,
    mut connection: Connection<crate::DATABASE>,
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
        
        // This code checks the user's login credentials and returns a user if the credentials are correct.
        let user: User = match match User::check_login_credentials(
            &mut *connection,
            form.user_id.clone(),
            form.password.clone(),
        )
        .await
        {
            Ok((password_is_right, user)) => {
                if password_is_right {
                    Ok(user)
                } else {
                    Err(())
                }
            }
            Err(_) => {
                Err(())
            }
        } {
            Ok(u) => u,
            Err(_) => {
                return Status::Forbidden;
            }
        };

        // IF SMTP Server is enabled
        if config.smtp.is_some() && user.additional_protection {
            //same thing happened here, had to use a useless match when I would normally just use a map_err...
            match email::send_login_code(&mut *connection, &config, &user).await {
                Ok(_) => (),
                Err(e) => {
                    error!("{e}");
                    return Status::InternalServerError;
                }
            }
            session
                .set(SessionToken::AwaitingConfirmation {
                    user_id: form.user_id.clone(),
                })
                .await
                .unwrap();
            return Status::Accepted;
        }

        if let Err(e) = session
            .set(SessionToken::LoggedIn {
                user_id: form.user_id.clone(),
            })
            .await
        {
            error!("{e}");
            return Status::InternalServerError;
        };
        Status::Ok
    }()
    .await;

    match code {
        //User Logged in successfully and can access his account.
        Status { code: 200 } => {
            return (
                Status::Ok,
                Template::render("auth/components/login-success", parameters),
            );
        }
        //User Logged in, but have to insert the email code
        Status { code: 202 } => {
            return (
                Status::Accepted,
                Template::render("auth/components/login-confirmation-code", parameters),
            );
        }
        //Wrong Password
        Status { code: 403 } => {
            return (
                Status::Forbidden,
                Template::render("auth/components/alert-wrong-password", parameters),
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
