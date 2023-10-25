use std::collections::HashMap;

use log::error;

use rocket::get;
use rocket::{form::Form, post, response::Redirect, FromForm, State};
use rocket_db_pools::Connection;

use super::check_recaptcha_token;
use rocket_dyn_templates::Template;
use rocket_recaptcha_v3::ReCaptcha;
use rocket_recaptcha_v3::ReCaptchaToken;
use strfmt::strfmt;

use crate::authentication::{Alert, CSRF};
use crate::users::UserAccountStatus;
use crate::{configuration, users::User};
use super::email::{self, Code};
use super::AuthParameters;

use super::SessionToken;

#[derive(FromForm, Debug)]
pub struct ConfirmationForm {
    code: String,
    password: String,
}

#[derive(FromForm, Debug)]
pub struct ResetForm {
    email: String,
    recaptcha_token: Option<ReCaptchaToken>,
}

#[get("/auth/reset")]
pub async fn page(
    recaptcha: &State<ReCaptcha>,
    _connection: Connection<crate::DATABASE>,
    config: &State<configuration::SharkNoteConfig>,
) -> Template {
    let mut template_args = AuthParameters::new(config, recaptcha);

    if config.auth.recaptcha {
        template_args.recaptcha_key = recaptcha.get_html_key_as_str().map(|a| a.to_string());
    }
    return Template::render("auth-reset", template_args);
}

#[post("/auth/reset-conf", data = "<form>")]
pub async fn confirmation(
    session: rocket_session_store::Session<'_, SessionToken>,
    form: Form<ConfirmationForm>,
    config: &State<configuration::SharkNoteConfig>,
    recaptcha: &State<ReCaptcha>,
    csrf: &State<CSRF>,
    mut connection: Connection<crate::DATABASE>,
) -> Result<Template, Redirect> {
    let mut parameters = AuthParameters::new(config,recaptcha);
    let session = SessionToken::init(&session, csrf).await;

    if let SessionToken::AwaitingConfirmation {user_id, csrf_token } = session {
        let mut user: User = User::get(&mut connection, user_id).await.unwrap();
        if let Ok(code) = Code::get(&mut connection, &user).await {
            if form.code == code.code.to_string() {
                user.set_status(&mut connection, UserAccountStatus::Normal)
                    .await
                    .unwrap();
                user.change_password(&mut connection, form.password.clone())
                    .await
                    .unwrap();

                parameters.alert = Some(super::Alert { alert_level: super::AlertLevel::Success, message: config.messages.account_login_success.clone()});
                parameters.final_button = Some(super::FinalButton {
                    href: "/editor".into(),
                    text: config.messages.account_login_link.clone(),
                });
                return Ok(Template::render("auth-base", parameters));
            }
        }
        parameters.alert = Some(super::Alert { alert_level: super::AlertLevel::Error, message: config.messages.confirmation_code_error.clone() });
        return Ok(Template::render("auth-conf", parameters));
    }
    //In case the user entered this page as a mistake.
    return Err(Redirect::to("/auth"));
}

#[post("/auth/reset", data = "<form>")]
pub async fn post(
    recaptcha: &State<ReCaptcha>,
    session: rocket_session_store::Session<'_, SessionToken>,
    form: Form<ResetForm>,
    config: &State<configuration::SharkNoteConfig>,

    csrf: &State<CSRF>,
    mut connection: Connection<crate::DATABASE>,
) -> Result<Template, Redirect> {
    let mut parameters = AuthParameters::new(config,recaptcha);
    parameters.recaptcha_key = recaptcha.get_html_key_as_str().map(|a| a.to_string());

    //MANAGES RECAPTCHA
    if let Err(template) = check_recaptcha_token(config, recaptcha, form.recaptcha_token.clone()).await {
        return Ok(template);
    }
    if let Ok(user) = User::get_from_email(&mut *connection, form.email.clone()).await {
        // IF SMTP Server is enabled
        if let Some(smtp) = &config.smtp {
            let code = Code::generate(&mut connection, &user).await.unwrap();
            let mut vars = HashMap::new();
            vars.insert(
                "display_program_name".to_string(),
                config.messages.display_program_name.clone(),
            );
            vars.insert("user_id".to_string(), user.user_id.clone());
            vars.insert("confirmation_code".to_string(), code.to_string());
            let email = email::send_email(
                smtp,
                user.email.clone(),
                strfmt(&config.messages.email_reset_title, &vars).unwrap(),
                strfmt(&config.messages.email_reset_text, &vars).unwrap(),
            );

            // Send the email
            if let Err(e) = email {
                error!("{e}");
                parameters.alert = Some(Alert{ alert_level: crate::authentication::AlertLevel::Error, message: config.messages.account_email_send_error.clone().into() });
                return Ok(Template::render("auth-panel", parameters));
            }
            session.set(SessionToken::AwaitingConfirmation {
                user_id: user.user_id.clone(),
                csrf_token: csrf.new_token()
            })
            .await
            .unwrap();
        }
    }
    parameters.alert = Some(Alert { alert_level: super::AlertLevel::Success, message: config.messages.reset_confirmation_code_info.clone() });
    return Ok(Template::render("auth-conf", parameters));
}
