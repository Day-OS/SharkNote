use std::{fs, path::Path};

use rocket::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Messages {
    pub display_program_name: String,
    pub email_registration_title: String,
    pub email_registration_text: String,
    pub account_creation_success: String,
    pub account_creation_error: String,
    pub account_email_send_error: String,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct SMTP {
    pub smtp_relay: String,
    pub smtp_username: String,
    pub smtp_password: String,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Auth {
    /// Makes so the only way to Register is if the user's email is allowed by a admin.
    pub invite_only: bool,
    /// If empty the feature will be turned off
    pub recaptcha: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct SharkNoteConfig {
    //pub secret_key: String,
    pub port: u16,
    pub auth: Auth,
    pub messages: Messages,
    pub smtp: Option<SMTP>,
}
impl Default for SharkNoteConfig {
    fn default() -> Self {
        SharkNoteConfig{
            port: 8000,
            smtp: None,
            auth: Auth { invite_only: false, recaptcha: false },
            messages: Messages { 
                account_creation_error: "Your account could not be created.\n Reason: {}".to_owned(),
                account_creation_success: "Your account has been created successfully!".to_owned(),
                display_program_name: "SharkNote".to_string(),
                email_registration_title: "{display_program_name} - Registration Confirmation".into(), 
                email_registration_text: r#"
                Dear {user_id},
                We are pleased to confirm your registration for {display_program_name}. Your commitment to joining our program is greatly appreciated, and we are excited to have you on board.

                To activate your account, please follow the provided link below:
                {confirmation_code}

                Upon clicking the link, your account will be enabled.

                Thank you for choosing {display_program_name}.

                Best regards,
                {display_program_name} Team"#.to_string(),
                account_email_send_error: "Could not send a email to the provided email.".to_string(),
            },
        }
    }
}
