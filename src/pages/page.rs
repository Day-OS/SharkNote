use crate::users::user::User;

use rocket::tokio;
//use crate::users::user::User;
use rocket_db_pools::sqlx;
use serde::Serialize;
use sqlx::Sqlite;

use super::PageStatus;

#[derive(sqlx::FromRow, Serialize)]
pub struct Page {
    pub page_id: String,
    pub status: PageStatus,
}
#[derive(sqlx::FromRow)]
pub struct PageUser {
    pub page_id: String,
    pub user_id: String,
    pub permission: super::Permission,
}


impl Page {
    pub async fn new(
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
        page_id: String,
        page_status: Option<PageStatus>,
    ) -> Result<Page, sqlx::Error> {
        tokio::fs::create_dir_all(format!("data/{}", page_id))
            .await
            .unwrap();
        let page_status = page_status.unwrap_or(PageStatus::Private);
        let page = sqlx::query_as::<_, Page>(
            "INSERT INTO page (page_id, status) VALUES (?1, ?2) RETURNING *;",
        )
        .bind(page_id.to_lowercase())
        .bind(page_status)
        .fetch_one(connection)
        .await?;
        Ok(page)
    }

    pub async fn delete(
        self: Self,
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM page WHERE page_id = ?1")
            .bind(self.page_id)
            .execute(connection)
            .await?;
        Ok(())
    }

    pub async fn get(
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
        page_id: String,
    ) -> Result<Page, sqlx::Error> {
        let page = sqlx::query_as::<_, Page>("SELECT * FROM page where page_id = ?1")
            .bind(page_id.to_lowercase())
            .fetch_one(connection)
            .await?;
        Ok(page)
    }

    pub async fn set_collaborator(
        self: &Self,
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
        user: &User,
        permission: super::Permission,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO page_user (page_id, user_id, permission) VALUES (?1, ?2 ,?3)")
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
        sqlx::query("DELETE FROM page_user WHERE page_id = ?1")
            .bind(self.page_id.clone())
            .execute(connection)
            .await?;
        Ok(())
    }

    pub async fn get_colaborators(
        self: &Self,
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
    ) -> Result<Vec<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM user where user_id in (SELECT user_id FROM page_user WHERE page_id = ?1)")
            .bind(self.page_id.clone())
            .fetch_all(connection)
            .await?;
        Ok(user)
    }

    pub async fn get_user_permission(
        self: &Self,
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
        user: User,
    ) -> Result<super::Permission, sqlx::Error> {
        let relation = sqlx::query_as::<_, PageUser>("SELECT * FROM page_user WHERE page_id = ?1 AND user_id = ?2")
            .bind(self.page_id.clone())
            .bind(user.user_id)
            .fetch_one(connection)
            .await?;
        Ok(relation.permission)
    }
}

