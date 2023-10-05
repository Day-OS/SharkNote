use rocket::{get, response::Redirect};

use super::SessionCookie;

#[get("/auth/logout")]
pub async fn confirmation(session: rocket_session_store::Session<'_, String>) -> Redirect {
    SessionCookie::set(&session, SessionCookie::Guest).await;
    Redirect::to("/auth")
}
