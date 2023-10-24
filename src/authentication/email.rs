use std::collections::HashMap;

use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, Message,
    SmtpTransport, Transport,
};
use rand::Rng;
use sqlx::Sqlite;
use strfmt::strfmt;

use crate::{configuration::{SMTP, self}, users::User};
pub fn send_email(
    smtp: &SMTP,
    destinatary: String,
    subject: String,
    body: String,
) -> Result<(), String> {
    let email = Message::builder()
        .from(format!("<{}>", smtp.smtp_username.clone()).parse().unwrap())
        .to(format!("<{}>", destinatary).parse().unwrap())
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body).map_err(|e| e.to_string())?;

    let creds = Credentials::new(smtp.smtp_username.clone(), smtp.smtp_password.clone());

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay(&smtp.smtp_relay)
        .map_err(|e| e.to_string())?
        .credentials(creds)
        .build();
    return mailer.send(&email).map(|_| ()).map_err(|e| e.to_string());
}
pub async fn _send_code(
    connection: &mut sqlx::SqliteConnection,
    config: &configuration::SharkNoteConfig,
    user: &User,
    subject: &String,
    body: &String,
) -> Result<(), String> {
    let code = Code::generate(connection, &user).await.unwrap();
        let mut vars = HashMap::new();
        vars.insert(
            "display_program_name".to_string(),
            config.messages.display_program_name.clone(),
        );
        vars.insert("user_id".to_string(), user.user_id.clone());
        vars.insert("confirmation_code".to_string(), code.to_string());
    let smtp = &config.smtp.clone().unwrap();
    send_email(
        smtp,
        user.email.clone(),
        strfmt(&subject, &vars).unwrap(),
        strfmt(&body, &vars).unwrap(),
    )

}

pub async fn send_login_code(
    connection: &mut sqlx::SqliteConnection,
    config: &configuration::SharkNoteConfig,
    user: &User) -> Result<(), String>{
    _send_code(connection, config, user, &config.messages.email_login_title, &config.messages.email_login_text).await

}



#[derive(sqlx::FromRow)]
pub struct Code {
    pub user_id: String,
    pub code: u32,
}
impl Code {
    pub async fn generate(
        connection: &mut sqlx::SqliteConnection,
        user: &User,
    ) -> Result<u32, sqlx::Error> {
        let code: u32 = rand::thread_rng().gen_range(0..999999);
        let _ = sqlx::query_as::<_, Code>(
            "INSERT INTO user_code (user_id, code) VALUES (?1, ?2) RETURNING *;",
        )
        .bind(user.user_id.clone())
        .bind(code)
        .fetch_one(connection)
        .await?;

        Ok(code)
    }

    pub async fn get(
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
        user: &User,
    ) -> Result<Self, sqlx::Error> {
        let code = sqlx::query_as::<_, Code>("SELECT * FROM user_code WHERE user_id = ?1;")
            .bind(user.user_id.clone())
            .fetch_one(connection)
            .await?;
        Ok(code)
    }

    pub async fn delete(
        self: Self,
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM user_code WHERE user_id = ?1;")
            .bind(self.user_id)
            .execute(connection)
            .await?;
        Ok(())
    }
}
