use rocket::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Messages {
    pub display_program_name: String,
    pub email_registration_title: String,
    pub email_registration_text: String,
    pub account_creation_success: String,
    pub account_creation_error: String,
    pub account_login_error: String,
    pub account_email_send_error: String,
    pub confirmation_code_error: String,
    pub confimation_code_info: String,
    pub email_login_title: String,
    pub email_login_text: String,
    pub account_login_success: String,
    pub account_creation_link: String,
    pub account_login_link: String,
    pub email_reset_text: String,
    pub email_reset_title: String,
    pub reset_confimation_code_info: String,
    pub login_confimation_code_info: String,
    pub not_invited: String,
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
                not_invited: "You were not invited. Ask someone for permission to join {display_program_name}.".to_string(),
                account_login_link: "Click Here to go to the editor".to_string(),
                account_login_success: "The confirmation code provided was right.".to_owned(),
                account_login_error: "The provided password is wrong or the account does not exist".to_owned(),
                account_creation_link: "Click Here to go back to the login page".to_string(),
                account_creation_success: "Your account has been created successfully!".to_owned(),
                account_creation_error: "Your account could not be created.\n Reason: {reason}".to_owned(),
                confirmation_code_error: "The provided code is not correct, try again.".to_string(),
                login_confimation_code_info: "You've received a confirmation code via email to log in into your account. Please check your inbox and enter the code below to complete the process.".to_string(),
                confimation_code_info: "You've received a confirmation code via email to activate your new account. Please check your inbox and enter the code below to complete the process.".to_string(),
                reset_confimation_code_info: "If your account exist, you received a confirmation code via email to reset your password. Please check your inbox and enter the code below to complete the process.".to_string(),
                account_email_send_error: "Could not send an email to the provided email.".to_string(),
                display_program_name: "SharkNote".to_string(),
                email_login_title: "{display_program_name} - Login Confirmation".into(), 
                email_login_text: r#"
                Dear {user_id},

                We are delighted to welcome you back to {display_program_name}. Your presence is highly valued, and we're thrilled to have you on board once again.

                To access your account, please enter the provided code on the login page:
                {confirmation_code}

                Thank you for continuing to choose {display_program_name}.

                Best regards,
                {display_program_name} Team"#.to_string(),
                email_registration_title: "{display_program_name} - Registration Confirmation".into(), 
                email_registration_text: r#"
                Dear {user_id},
                We are pleased to confirm your registration for {display_program_name}. We are excited to have you on board.

                To activate your account, please insert the provided code in the confirmation page:
                {confirmation_code}

                Thank you for choosing {display_program_name}.

                Best regards,
                {display_program_name} Team"#.to_string(),
                email_reset_title: "{display_program_name} - Password Reset Confirmation".into(), 
                email_reset_text: r#"
                Dear {user_id},
                We are pleased to confirm your request for a password reset for your {display_program_name} account. We are here to assist you in this process.

                To reset your password, please insert the provided code in the confirmation page:
                {confirmation_code}

                Thank you for choosing {display_program_name}.

                Best regards,
                {display_program_name} Team"#.to_string(),
            },
        }
    }
}
