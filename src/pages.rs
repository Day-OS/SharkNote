use crate::users::User;

use rocket::tokio;
//use crate::users::user::User;
use rocket_db_pools::sqlx;
use serde::Serialize;
use sqlx::Sqlite;
pub mod files;
pub mod permissions;


#[derive(sqlx::FromRow, Serialize)]
pub struct Page {
    pub page_id: String,
    pub status: PageStatus,
}

#[derive(sqlx::Type, Serialize)]
pub enum PageStatus {
    Public,   //Anyone can access it from anywhere. It also shows up on search websites.
    LinkOnly, //Anyone with a link can acess it. It does not show up on search websites.
    Private,  //Only a user that is logged into the website
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
}

