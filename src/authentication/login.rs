use log::error;

use log::info;
use rocket::http::Status;
use rocket::{form::Form, post, response::Redirect, FromForm, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use rocket_recaptcha_v3::ReCaptcha;
use rocket_recaptcha_v3::ReCaptchaToken;

use crate::authentication::CSRF;
use crate::{configuration, users::User};

use super::check_recaptcha_token;
use super::email;
use super::AuthParameters;

use super::email::Code;
use super::SessionToken;

#[derive(FromForm, Debug)]
pub(crate) struct ConfirmationForm {
    code: String,
}

#[derive(FromForm, Debug)]
pub(crate) struct LoginForm {
    user_id: String,
    password: String,
    recaptcha_token: Option<ReCaptchaToken>,
}

#[post("/auth/login-conf", data = "<form>")]
pub async fn confirmation(
    session: rocket_session_store::Session<'_, SessionToken>,
    recaptcha: &State<ReCaptcha>,
    form: Form<ConfirmationForm>,
    config: &State<configuration::SharkNoteConfig>,
    mut connection: Connection<crate::DATABASE>,
    csrf: &State<CSRF>,
) -> Result<Template, Redirect> {
    todo!();
    let mut parameters = AuthParameters::new(config, recaptcha);
    let session = SessionToken::init(&session, csrf).await;

    if let SessionToken::AwaitingConfirmation { user_id , csrf_token:_} = session {
        let user: User = User::get(&mut connection, user_id).await.unwrap();
        if let Ok(code) = Code::get(&mut connection, &user).await {
            if form.code == code.code.to_string() {
                parameters.alert = Some(super::Alert {
                    alert_level: super::AlertLevel::Success,
                    message: config.messages.account_login_success.clone(),
                });
                parameters.final_button = Some(super::FinalButton {
                    href: "/editor".into(),
                    text: config.messages.account_login_link.clone(),
                });
                return Ok(Template::render("auth/base", parameters));
            }
        }
        parameters.alert = Some(super::Alert {
            alert_level: super::AlertLevel::Error,
            message: config.messages.confirmation_code_error.clone(),
        });
        return Ok(Template::render("auth-conf", parameters));
    }
    //In case the user entered this page as a mistake.
    return Err(Redirect::to("/auth"));
}

#[post("/auth/login", data = "<form>")]
pub async fn post(
    recaptcha: &State<ReCaptcha>,
    session: rocket_session_store::Session<'_, SessionToken>,
    form: Form<LoginForm>,
    config: &State<configuration::SharkNoteConfig>,
    mut connection: Connection<crate::DATABASE>,
    csrf: &State<CSRF>,
) -> (Status, Template) {
    let mut parameters = AuthParameters::new(config, recaptcha);
    parameters.recaptcha_key = recaptcha.get_html_key_as_str().map(|a| a.to_string());

    let code = async move || -> Status {
        if let Err(_) = check_recaptcha_token(config, recaptcha, form.recaptcha_token.clone()).await
        {
            return Status::Forbidden;
        };

        //Had to use two matches because the compiler was drunk and wouldn't help. It was giving me an error that the parameter variable was moved inside the map_err closure. It shouldn't happen because I was using the ? macro. I'm actually pissed off by this...
        let user = match match User::check_login_credentials(
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
            Err(e) => {
                info!("{e} | In a login attempt.");
                Err(())
            }
        } {
            Ok(u) => u,
            Err(_) => {
                return Status::Forbidden;
            }
        };

        // IF SMTP Server is enabled
        if config.smtp.is_some() {
            //same thing happened here, had to use a useless match when I would normally just use a map_err...
            match email::send_login_code(&mut *connection, &config, &user).await {
                Ok(_) => (),
                Err(e) => {
                    error!("{e}");
                    return Status::InternalServerError;
                }
            }
            session.set(SessionToken::AwaitingConfirmation {
                user_id: form.user_id.clone(),
                csrf_token: csrf.new_token()
            })
            .await
            .unwrap();
            return Status::Accepted;
        }

        if let Err(_) = session.set(SessionToken::LoggedIn {
            user_id: form.user_id.clone(),
            csrf_token: csrf.new_token(),
        })
        .await
        {
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
                Template::render("auth/components/login-register-forms", parameters),
            );
        }
        Status { code: 500 } | _ => {
            return (
                Status::InternalServerError,
                Template::render("auth/components/login-register-forms", parameters),
            );
        }
    }
}
