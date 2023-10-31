use log::error;
use rocket::get;
use rocket::http::Status;
use rocket::{form::Form, post, FromForm, State};
use rocket_csrf_token::CsrfToken;
use rocket_db_pools::Connection;

use super::check_recaptcha_token;
use rocket_dyn_templates::Template;
use rocket_recaptcha_v3::ReCaptcha;
use rocket_recaptcha_v3::ReCaptchaToken;

use super::email::{self, Code};
use super::AuthParameters;
use crate::users::UserAccountStatus;
use crate::{configuration, users::User};

use super::SessionToken;

#[get("/auth/reset")]
pub async fn page(
    recaptcha: &State<ReCaptcha>,
    _connection: Connection<crate::DATABASE>,
    config: &State<configuration::SharkNoteConfig>,
) -> Template {
    let mut template_args = AuthParameters::new(config).add_recaptcha(recaptcha);

    if config.auth.recaptcha {
        template_args.recaptcha_key = recaptcha.get_html_key_as_str().map(|a| a.to_string());
    }
    return Template::render("auth-reset", template_args);
}

#[derive(FromForm, Debug)]
pub struct ConfirmationForm {
    code: String,
    password: String,
    csrf_token: String,
    recaptcha_token: Option<ReCaptchaToken>,
}

#[post("/auth/reset/confirmation", data = "<form>")]
pub async fn confirmation(
    session: rocket_session_store::Session<'_, SessionToken>,
    form: Form<ConfirmationForm>,
    config: &State<configuration::SharkNoteConfig>,
    recaptcha: &State<ReCaptcha>,
    csrf: CsrfToken,
    mut connection: Connection<crate::DATABASE>,
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
            let mut user: User = User::get(&mut connection, user_id.clone()).await.unwrap();
            if let Ok(code) = Code::get(&mut connection, &user).await {
                if form.code == code.code.to_string() {
                    user.set_status(&mut connection, UserAccountStatus::Normal)
                        .await
                        .unwrap();
                    user.change_password(&mut connection, form.password.clone())
                        .await
                        .unwrap();
                    session.set(SessionToken::LoggedIn { user_id: user_id }).await.unwrap();
                    return Status::Ok;
                }
            }
        }

        Status::Forbidden
    }()
    .await;
    match code {
        //User Logged in successfully and can access his account.
        Status { code: 200 } => {
            return (
                Status::Ok,
                Template::render("auth/components/reset-success", parameters),
            );
        }
        Status { code: 403 } | _ => {
            return (
                Status::Forbidden,
                Template::render("auth/components/alert-wrong-code", parameters),
            );
        }
    }
}

#[derive(FromForm, Debug)]
pub struct ResetForm {
    email: String,
    csrf_token: String,
    recaptcha_token: Option<ReCaptchaToken>,
}

#[post("/auth/reset", data = "<form>")]
pub async fn post(
    mut connection: Connection<crate::DATABASE>,
    recaptcha: &State<ReCaptcha>,
    csrf: CsrfToken,
    session: rocket_session_store::Session<'_, SessionToken>,
    form: Form<ResetForm>,
    config: &State<configuration::SharkNoteConfig>,
) -> (Status, Template) {
    let parameters = AuthParameters::new(config);

    let code = async move || -> Status {
        if let Err(st) = super::check_csrf(&form.csrf_token, &csrf) {
            return st;
        };
        if let Err(st) = check_recaptcha_token(config, recaptcha, &form.recaptcha_token).await {
            return st;
        };

        if let Ok(user) = User::get_from_email(&mut *connection, form.email.clone()).await {
            if config.smtp.is_some() {
                if let Err(e) = email::send_reset_code(&mut connection, config, &user).await {
                    error!("{e}");
                    return Status::InternalServerError;
                }
                if let Err(e) = session
                    .set(SessionToken::AwaitingConfirmation {
                        user_id: user.user_id.clone(),
                    })
                    .await
                {
                    error!("{e}");
                    return Status::InternalServerError;
                };
                user.set_status(&mut connection, UserAccountStatus::PasswordRecovery)
                .await
                .unwrap();
                return Status::Accepted;
            }
            return Status::InternalServerError;
        }
        Status::BadRequest
    }()
    .await;

    match code {
        //User Logged in successfully and can access his account.
        Status { code: 202 } => {
            return (
                Status::Accepted,
                Template::render("auth/components/reset-confirmation-code", parameters),
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

#[derive(FromForm)]
struct MainFormsForm {
    csrf_token: String,
}

///The first screen that appears containing the login and the register forms.
#[post("/auth/components/reset", data = "<form>")]
pub async fn component(
    form: Form<MainFormsForm>,
    csrf: CsrfToken,
    config: &State<configuration::SharkNoteConfig>,
) -> (Status, Option<Template>) {
    if let Err(status) = super::check_csrf(&form.csrf_token, &csrf) {
        return (status, None);
    };
    let template_args = AuthParameters::new(config);

    return (
        Status::Ok,
        Some(Template::render(
            "auth/components/reset-form",
            template_args,
        )),
    );
}
