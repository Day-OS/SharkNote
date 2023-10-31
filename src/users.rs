pub mod invite;
use crate::pages::{permissions::Permission, Page};
use log::error;
use rand::{RngCore, Rng, distributions};
use regex::Regex;
use rocket::http::Status;
use sha256::digest;

const REG_USERNAME: &str = "^[a-z0-9]{3,20}$";
const REG_PASSWORD: &str = r#"[A-Za-zÀ-ÖØ-öø-ÿÇç\d@#$%&*_!?\-]{8,}$"#;
const REG_EMAIL: &str = r#"^([a-zA-Z0-9_\-\.]+)@([a-zA-Z0-9_\-]+)(\.[a-zA-Z]{2,5}){1,2}$"#;

#[derive(sqlx::FromRow)]
pub struct User {
    pub user_id: String,
    pub password: String,
    pub salt: String,
    pub email: String,
    pub display_name: String,
    pub configuration_json: String,
    pub is_program_admin: u8,
    pub account_status: UserAccountStatus,
    pub additional_protection: bool,

}
#[derive(sqlx::Type)]
pub enum UserAccountStatus {
    Normal,
    RegistrationPending,
    PasswordRecovery,
    Banned,
}

fn generate_salt() -> String{
    rand::thread_rng()
    .sample_iter(distributions::Alphanumeric)
    .take(10)
    .map(char::from)
    .collect()
}

fn sha256_password(salt: String, password: String) -> String {
    digest(format!("{}{}", salt, password))
}

impl User {
    pub async fn new(
        connection: &mut sqlx::SqliteConnection,
        user_id: String,
        password: String,
        email: String,
        account_status: UserAccountStatus,
    ) -> Result<Self, Status> {
        
        let input_valid = || -> Result<bool, regex::Error> {
            Ok( 
                Regex::new(REG_USERNAME)?.is_match(&user_id) &&
                Regex::new(REG_PASSWORD)?.is_match(&password) &&
                Regex::new(REG_EMAIL)?.is_match(&email)
            )
        }().map_err(|e| {
            error!("Regex verification failed. | {e:?}");
            Status::InternalServerError
        })?;
        if !input_valid { 
            error!("Inputs are invalid.");
            return Err(Status::Forbidden) }
        
        let salt = generate_salt();
        let hash_password = sha256_password(salt.clone(), password);
        let user = sqlx::query_as::<_, Self>("INSERT INTO user (user_id, password, is_program_admin, email, account_status, additional_protection, salt) VALUES (?1, ?2, 0, ?3, ?4, 0, ?5) RETURNING *; DELETE FROM user_invited WHERE user_id = ?3;")
        .bind(user_id)
        .bind(hash_password)
        .bind(email)
        .bind(account_status)
        .bind(salt)
        .fetch_one(connection)
        .await.map_err(|e| {error!("{e}"); Status::InternalServerError})?;

        Ok(user)
    }

    pub async fn get(
        connection: &mut sqlx::SqliteConnection,
        user_id: String,
    ) -> Result<Self, sqlx::Error> {
        let user = sqlx::query_as::<_, Self>("SELECT * FROM user where user_id = ?1;")
            .bind(user_id)
            .fetch_one(connection)
            .await?;
        Ok(user)
    }
    pub async fn get_from_email(
        connection: &mut sqlx::SqliteConnection,
        email: String,
    ) -> Result<Self, sqlx::Error> {
        let user = sqlx::query_as::<_, Self>("SELECT * FROM user where email = ?1;")
            .bind(email)
            .fetch_one(connection)
            .await?;
        Ok(user)
    }

    pub async fn delete(
        self: Self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM user WHERE user_id = ?1")
            .bind(self.user_id)
            .execute(connection)
            .await?;
        Ok(())
    }

    pub async fn set_status(
        self: &Self,
        connection: &mut sqlx::SqliteConnection,
        status: UserAccountStatus,
    ) -> Result<&Self, sqlx::Error> {
        sqlx::query("UPDATE user SET account_status = ?1 WHERE user_id = ?2;")
            .bind(status)
            .bind(self.user_id.clone())
            .execute(connection)
            .await?;
        Ok(self)
    }
    pub async fn set_additional_protection(
        self: &Self,
        connection: &mut sqlx::SqliteConnection,
        protection: bool,
    ) -> Result<&Self, sqlx::Error> {
        sqlx::query("UPDATE user SET additional_protection = ?1 WHERE user_id = ?2;")
            .bind(protection)
            .bind(self.user_id.clone())
            .execute(connection)
            .await?;
        Ok(self)
    }

    pub async fn set_admin(
        self: &Self,
        connection: &mut sqlx::SqliteConnection,
        b: bool,
    ) -> Result<&Self, sqlx::Error> {
        sqlx::query("UPDATE user SET is_program_admin = ?1 WHERE user_id = ?2;")
            .bind(b as u8)
            .bind(self.user_id.clone())
            .execute(connection)
            .await?;
        Ok(self)
    }

    pub async fn change_password(
        self: &mut Self,
        connection: &mut sqlx::SqliteConnection,
        password: String,
    ) -> Result<&Self, Status> {

        let input_valid = || -> Result<bool, regex::Error> {
            Ok( Regex::new(REG_PASSWORD)?.is_match(&password))
        }().map_err(|e| {
            error!("Regex verification failed. | {e:?}");
            Status::InternalServerError
        })?;

        if !input_valid{
            return Err(Status::Forbidden);
        }

        let hash_password = sha256_password(generate_salt(), password);
        sqlx::query("UPDATE user SET password = ?1 WHERE user_id = ?2")
            .bind(hash_password)
            .bind(self.user_id.clone())
            .execute(connection)
            .await.map_err(|e| {
                error!("{e}"); 
                Status::InternalServerError
            })?;
        Ok(self)
    }

    pub async fn check_login_credentials(
        connection: &mut sqlx::SqliteConnection,
        user_id: String,
        password: String,
    ) -> Result<(bool, User), sqlx::Error> {
        let user = User::get(connection, user_id).await?;
        Ok((sha256_password(user.salt.clone(), password) == user.password.clone(), user))
    }

    pub async fn get_modifiable_pages(
        self: &Self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<Vec<Page>, sqlx::Error> {
        sqlx::query_as::<_, Page>("SELECT * FROM page WHERE page_id IN (SELECT page_id FROM permission WHERE user_id = ?1 AND permission = ?2);")
        .bind(self.user_id.clone())
        .bind(Permission::ModifyContent)
        .fetch_all(connection)
        .await
    }
}
