use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, Message,
    SmtpTransport, Transport,
};

use crate::configuration::SMTP;
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
        .body(body)
        .unwrap();

    let creds = Credentials::new(smtp.smtp_username.clone(), smtp.smtp_password.clone());

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay(&smtp.smtp_relay)
        .unwrap()
        .credentials(creds)
        .build();
    return mailer.send(&email).map(|_| ()).map_err(|e| e.to_string());
}
