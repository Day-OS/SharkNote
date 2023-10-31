use super::AuthParameters;
use super::SessionToken;
use crate::configuration;
use crate::users::User;
use rocket::form::Form;
use rocket::http::Status;
use rocket::{post, FromForm, State};
use rocket_csrf_token::CsrfToken;
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

#[derive(FromForm)]
struct MainFormsForm {
    csrf_token: String,
}

///The first screen that appears containing the login and the register forms.
#[post("/auth/components/main_forms", data = "<form>")]
pub async fn component(
    session: rocket_session_store::Session<'_, SessionToken>,
    form: Form<MainFormsForm>,
    csrf: CsrfToken,
    mut connection: Connection<crate::DATABASE>,
    _connection: Connection<crate::DATABASE>,
    config: &State<configuration::SharkNoteConfig>,
) -> (Status, Option<Template>) {
    if let Err(status) = super::check_csrf(&form.csrf_token, &csrf) {
        return (status, None);
    };

    let template_args = AuthParameters::new(config);

    match SessionToken::init(&session).await {
        SessionToken::LoggedIn { user_id: _ } => {
            return (
                Status::Ok,
                Some(Template::render(
                    "auth/components/already-logged-in",
                    template_args,
                )),
            );
        }
        SessionToken::AwaitingConfirmation { user_id } => {
            let user = User::get(&mut connection, user_id.clone()).await.unwrap();
            match user.account_status {
            crate::users::UserAccountStatus::Normal => {
                return (
                    Status::Ok,
                    Some(Template::render(
                        "auth/components/login-confirmation-code",
                        template_args,
                    )),
                );
            },
            crate::users::UserAccountStatus::RegistrationPending => {
                return (
                    Status::Ok,
                    Some(Template::render(
                        "auth/components/register-confirmation-code",
                        template_args,
                    )),
                );
            },
            crate::users::UserAccountStatus::PasswordRecovery => {
                return (
                    Status::Ok,
                    Some(Template::render(
                        "auth/components/reset-confirmation-code",
                        template_args,
                    )),
                );
            },
            crate::users::UserAccountStatus::Banned => {
                return (
                    Status::Forbidden,
                    Some(Template::render(
                        "auth/components/alert-not_authorized",
                        template_args,
                    )),
                );
            },
}
            
        }
        SessionToken::Guest => {
            return (
                Status::Ok,
                Some(Template::render(
                    "auth/components/main-forms",
                    template_args,
                )),
            );
        }
    }
}
