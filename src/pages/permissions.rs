use crate::{authentication::SessionToken, users::User};
use rocket_csrf_token::CsrfToken;
use rocket_db_pools::sqlx;
use sqlx::Sqlite;

use rocket::{http::Status, State};
use rocket_session_store::Session;

use super::Page;

#[derive(sqlx::FromRow)]
pub struct Permissions {
    pub page_id: String,
    pub user_id: String,
    pub permission: Permission,
}

#[derive(sqlx::Type, Clone)]
pub enum Permission {
    DeletePage,
    ModifyContent,
    DeleteComments,
    SeePrivate,
}

impl super::Page {
    pub async fn get_user_permissions(
        self: &Self,
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
        user: &User,
    ) -> Result<Vec<Permission>, sqlx::Error> {
        let relation = sqlx::query_as::<_, Permissions>(
            "SELECT * FROM permission WHERE page_id = ?1 AND user_id = ?2",
        )
        .bind(self.page_id.clone())
        .bind(user.user_id.clone())
        .fetch_all(connection)
        .await?;
        Ok(relation.into_iter().map(|perm| perm.permission).collect())
    }

    pub async fn user_has_permissions(
        self: &Self,
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
        user: &User,
        needed_permissions: Vec<Permission>,
    ) -> Result<bool, sqlx::Error> {
        let user_permissions = self.get_user_permissions(connection, user).await?;

        let mut matched_permissions: usize = 0;
        for needed_permission in &needed_permissions {
            for user_permission in &user_permissions {
                if std::mem::discriminant(user_permission)
                    == std::mem::discriminant(needed_permission)
                {
                    matched_permissions += 1;
                }
            }
        }
        Ok(matched_permissions == needed_permissions.len())
    }

    pub async fn user_has_permission(
        self: &Self,
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
        user: &User,
        permission_needed: Permission,
    ) -> Result<bool, sqlx::Error> {
        self.user_has_permissions(connection, user, vec![permission_needed])
            .await
    }

    pub async fn set_permission(
        self: &Self,
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
        user: &User,
        permission: Permission,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO permission (page_id, user_id, permission) VALUES (?1, ?2 ,?3)")
            .bind(self.page_id.clone())
            .bind(user.user_id.clone())
            .bind(permission)
            .execute(connection)
            .await?;
        Ok(())
    }

    pub async fn remove_owner(
        self: &Self,
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM permission WHERE page_id = ?1")
            .bind(self.page_id.clone())
            .execute(connection)
            .await?;
        Ok(())
    }

    pub async fn get_colaborators(
        self: &Self,
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
    ) -> Result<Vec<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM user where user_id in (SELECT user_id FROM permission WHERE page_id = ?1)")
            .bind(self.page_id.clone())
            .fetch_all(connection)
            .await?;
        Ok(user)
    }
}

pub async fn get_page_if_allowed(
    connection: &mut sqlx::pool::PoolConnection<Sqlite>,
    page_id: &String,
    session: &Session<'_, SessionToken>,
    mut required_perms: Vec<Permission>,
    csrf: CsrfToken,
) -> Result<Page, Status> {
    let page = Page::get(connection, page_id.to_string())
        .await
        .map_err(|_| Status::NotFound)?;
    if required_perms.is_empty() {
        return Ok(page);
    }

    //In some cases the needed permission does not matter, this will be evaluated below
    //In this case, if the page is not private, it removes the permission necessity.
    match page.status {
        super::PageStatus::Private => {}
        _ => {
            for (i, p) in required_perms.clone().into_iter().enumerate() {
                if let Permission::SeePrivate = p {
                    required_perms.remove(i);
                }
            }
        }
    }

    //Then finally check if the user has it
    if let SessionToken::LoggedIn { user_id} = SessionToken::init(&session).await {
        let user = User::get(connection, user_id).await.map_err(|e| {
            log::error!("{e}");
            Status::InternalServerError
        })?;

        if page
            .user_has_permissions(connection, &user, required_perms)
            .await
            .map_err(|e| {
                log::error!("{e}");
                Status::InternalServerError
            })?
        {
            return Ok(page);
        };
    }
    return Err(Status::Unauthorized);
}
