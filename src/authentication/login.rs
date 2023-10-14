use log::error;

use rocket::{form::Form, post, response::Redirect, FromForm, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use rocket_recaptcha_v3::ReCaptcha;
use rocket_recaptcha_v3::ReCaptchaToken;
use std::collections::HashMap;
use strfmt::strfmt;

use crate::users::code::Code;
use crate::{configuration, users::User};

use super::check_recaptcha;
use super::email;
use super::AuthParameters;

use super::SessionCookie;

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
    session: rocket_session_store::Session<'_, String>,
    form: Form<ConfirmationForm>,
    config: &State<configuration::SharkNoteConfig>,
    mut connection: Connection<crate::DATABASE>,
) -> Result<Template, Redirect> {
    let mut parameters = AuthParameters::default();
    let session = SessionCookie::get(&session).await;

    if let SessionCookie::AwaitingConfirmation { user_id } = session {
        let user: User = User::get(&mut connection, user_id).await.unwrap();
        if let Ok(code) = Code::get(&mut connection, &user).await {
            if form.code == code.code.to_string() {
                parameters.alert_level = Some("success".into());
                parameters.message = Some(config.messages.account_login_success.clone());
                parameters.final_button = Some(super::FinalButton {
                    href: "/editor".into(),
                    text: config.messages.account_login_link.clone(),
                });
                return Ok(Template::render("auth-base", parameters));
            }
        }
        parameters.mode = Some("login".into());
        parameters.alert_level = Some("error".into());
        parameters.message = Some(config.messages.confirmation_code_error.clone());
        return Ok(Template::render("auth-conf", parameters));
    }
    //In case the user entered this page as a mistake.
    return Err(Redirect::to("/auth"));
}

#[post("/auth/login", data = "<form>")]
pub async fn post(
    recaptcha: &State<ReCaptcha>,
    session: rocket_session_store::Session<'_, String>,
    form: Form<LoginForm>,
    config: &State<configuration::SharkNoteConfig>,
    mut connection: Connection<crate::DATABASE>,
) -> Result<Template, Redirect> {
    let mut parameters = AuthParameters::default();
    parameters.recaptcha = recaptcha.get_html_key_as_str().map(|a| a.to_string());

    //MANAGES RECAPTCHA
    if let Err(template) = check_recaptcha(recaptcha, form.recaptcha_token.clone().unwrap()).await {
        return Ok(template);
    }

    let password_is_right: bool = match User::check_login_credentials(
        &mut *connection,
        form.user_id.clone(),
        form.password.clone(),
    )
    .await
    {
        Ok(b) => b,
        Err(_e) => {
            parameters.alert_level = Some("error".into());
            parameters.message = Some(config.messages.account_login_error.clone());
            return Ok(Template::render("auth-panel", parameters));
        }
    };
    if password_is_right == false {
        parameters.alert_level = Some("error".into());
        parameters.message = Some(config.messages.account_login_error.clone());
        return Ok(Template::render("auth-panel", parameters));
    }

    // IF SMTP Server is enabled
    if let Some(smtp) = &config.smtp {
        let user = User::get(&mut connection, form.user_id.clone())
            .await
            .unwrap();
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
            user.email.clone(),
            strfmt(&config.messages.email_login_title, &vars).unwrap(),
            strfmt(&config.messages.email_login_text, &vars).unwrap(),
        );

        // Send the email
        if let Err(e) = email {
            error!("{e}");
            parameters.alert_level = Some("error".into());
            parameters.message = Some(config.messages.account_email_send_error.clone().into());
            return Ok(Template::render("auth-panel", parameters));
        }
        SessionCookie::set(
            &session,
            SessionCookie::AwaitingConfirmation {
                user_id: form.user_id.clone(),
            },
        )
        .await
        .unwrap();
    }
    parameters.mode = Some("login".into());
    parameters.alert_level = Some("success".into());
    parameters.message = Some(config.messages.login_confimation_code_info.clone());
    return Ok(Template::render("auth-conf", parameters));
}
