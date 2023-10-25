use super::Alert;
use super::check_recaptcha_token;
use super::email;
use super::AuthParameters;
use super::email::Code;
use crate::authentication::CSRF;
use crate::users::invite;
use crate::users::UserAccountStatus;
use crate::{configuration, users::User};
use log::error;
use rocket::serde::json::Json;
use rocket::{form::Form, post, response::Redirect, FromForm, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use rocket_recaptcha_v3::ReCaptcha;
use rocket_recaptcha_v3::ReCaptchaToken;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strfmt::strfmt;

use super::SessionToken;

#[derive(FromForm, Debug)]
pub(crate) struct ConfirmationForm {
    code: String,
}

#[derive(FromForm, Debug)]
pub(crate) struct RegistrationForm {
    pub user_id: String,
    pub password: String,
    pub email: String,
    pub recaptcha_token: Option<ReCaptchaToken>,
}

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

#[post("/auth/register-conf", data = "<form>")]
pub async fn confirmation(
    recaptcha: &State<ReCaptcha>,
    session: rocket_session_store::Session<'_, SessionToken>,
    form: Form<ConfirmationForm>,
    config: &State<configuration::SharkNoteConfig>,
    mut connection: Connection<crate::DATABASE>,
    csrf: &State<CSRF>,
) -> Result<Template, Redirect> {
    let mut parameters = AuthParameters::new(config, recaptcha);
    let session = SessionToken::init(&session, csrf).await;

    if let SessionToken::AwaitingConfirmation { user_id, csrf_token } = session {
        let user: User = User::get(&mut connection, user_id).await.unwrap();
        if let Ok(code) = Code::get(&mut connection, &user).await {
            if form.code == code.code.to_string() {
                code.delete(&mut connection).await.unwrap();
                user.set_status(&mut connection, UserAccountStatus::Normal)
                    .await
                    .unwrap();
                parameters.alert = Some(Alert { alert_level: super::AlertLevel::Success, message: config.messages.account_creation_success.clone() });
                parameters.final_button = Some(super::FinalButton {
                    href: "/auth".into(),
                    text: config.messages.account_creation_link.clone(),
                });
                return Ok(Template::render("auth-base", parameters));
            }
        }
        parameters.alert = Some(Alert{ alert_level: super::AlertLevel::Error, message: config.messages.confirmation_code_error.clone()});
        return Ok(Template::render("auth-conf", parameters));
    }
    //In case the user entered this page as a mistake.
    return Err(Redirect::to("/auth"));
}

#[post("/auth/register", data = "<form>")]
pub async fn post(
    recaptcha: &State<ReCaptcha>,
    session: rocket_session_store::Session<'_, SessionToken>,
    form: Form<RegistrationForm>,
    config: &State<configuration::SharkNoteConfig>,
    mut connection: Connection<crate::DATABASE>,
    csrf: &State<CSRF>,
) -> Result<Template, Redirect> {
    let mut parameters = AuthParameters::new(config,recaptcha);
    parameters.recaptcha_key = recaptcha.get_html_key_as_str().map(|a| a.to_string());

    //MANAGES RECAPTCHA
    if let Err(template) = check_recaptcha_token(config, recaptcha, form.recaptcha_token.clone()).await {
        return Ok(template);
    }

    if config.auth.invite_only {
        let mut vars = HashMap::new();
        vars.insert(
            "display_program_name".to_string(),
            config.messages.display_program_name.clone(),
        );
        if !invite::Invite::is_email_invited(&mut *connection, form.email.clone()).await {
            parameters.alert = Some(Alert { alert_level: super::AlertLevel::Error, message: strfmt(&config.messages.not_invited, &vars).unwrap() });
            return Ok(Template::render("auth-panel", parameters));
        };
    }

    //CHECKS IF THE CREATION OF THE USER WENT WELL
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
        let mut hash = HashMap::new();
        hash.insert("reason".to_string(), e.to_string());
        parameters.alert = Some(Alert { alert_level: crate::authentication::AlertLevel::Error, message: strfmt(&config.messages.account_creation_error.clone(), &hash).unwrap() });
        return Ok(Template::render("auth-panel", parameters));
    }
    let user = user_result.unwrap();

    // IF SMTP Server is enabled
    if let Some(smtp) = &config.smtp {
        let code = Code::generate(&mut connection, &user).await.unwrap();
        let mut vars = HashMap::new();
        vars.insert(
            "display_program_name".to_string(),
            config.messages.display_program_name.clone(),
        );
        vars.insert("user_id".to_string(), form.user_id.clone());
        vars.insert("confirmation_code".to_string(), code.to_string());
        let email = email::send_email(
            smtp,
            form.email.clone(),
            strfmt(&config.messages.email_registration_title, &vars).unwrap(),
            strfmt(&config.messages.email_registration_text, &vars).unwrap(),
        );

        // Send the email
        if let Err(e) = email {
            error!("{e}");
            parameters.alert = Some(Alert { alert_level: crate::authentication::AlertLevel::Error, message: config.messages.account_email_send_error.clone().into() });
            return Ok(Template::render("auth-panel", parameters));
        }
        session.set(SessionToken::AwaitingConfirmation {
            user_id: form.user_id.clone(),
            csrf_token: csrf.new_token()
        })
        .await
        .unwrap();
    }
    parameters.alert = Some(Alert{ alert_level: super::AlertLevel::Success, message: config.messages.confirmation_code_info.clone()});
    return Ok(Template::render("auth-conf", parameters));
}
