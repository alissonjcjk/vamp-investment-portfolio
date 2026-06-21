use axum::{extract::FromRequestParts, http::request::Parts};
use sqlx::PgPool;
use std::convert::Infallible;

use crate::{
    app::AppState,
    models::{Asset, UserRecord},
};

pub struct Repository {
    db: PgPool,
}

impl Repository {
    pub async fn list_assets_by_user(&self, user_id: i64) -> sqlx::Result<Vec<Asset>> {
        sqlx::query_as!(
            Asset,
            "SELECT * FROM assets WHERE user_id = $1 ORDER BY created_at DESC",
            user_id
        )
        .fetch_all(&self.db)
        .await
    }

    pub async fn create_asset(
        &self,
        user_id: i64,
        name: String,
        quantity: f64,
        unit_value: f64,
    ) -> sqlx::Result<Asset> {
        sqlx::query_as!(
            Asset,
            "INSERT INTO assets (user_id, name, quantity, unit_value) VALUES ($1, $2, $3, $4) RETURNING *",
            user_id,
            name,
            quantity,
            unit_value
        )
        .fetch_one(&self.db)
        .await
    }

    pub async fn update_asset(
        &self,
        asset_id: i64,
        user_id: i64,
        name: Option<String>,
        quantity: Option<f64>,
        unit_value: Option<f64>,
    ) -> sqlx::Result<Option<Asset>> {
        sqlx::query_as!(
            Asset,
            r#"
            UPDATE assets
            SET
                name = COALESCE($1, name),
                quantity = COALESCE($2, quantity),
                unit_value = COALESCE($3, unit_value),
                updated_at = now()
            WHERE id = $4 AND user_id = $5
            RETURNING *
            "#,
            name,
            quantity,
            unit_value,
            asset_id,
            user_id
        )
        .fetch_optional(&self.db)
        .await
    }

    pub async fn delete_asset(&self, asset_id: i64, user_id: i64) -> sqlx::Result<u64> {
        let result = sqlx::query!(
            "DELETE FROM assets WHERE id = $1 AND user_id = $2",
            asset_id,
            user_id
        )
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn add_user(&self, username: &str, password_hash: &str) -> sqlx::Result<UserRecord> {
        sqlx::query_as!(
            UserRecord,
            "INSERT INTO users (username, password_hash) VALUES ($1, $2) RETURNING id, username, password_hash",
            username,
            password_hash
        )
        .fetch_one(&self.db)
        .await
    }

    pub async fn get_user_by_name(&self, username: &str) -> sqlx::Result<Option<UserRecord>> {
        sqlx::query_as!(
            UserRecord,
            "SELECT id, username, password_hash FROM users WHERE username = $1",
            username
        )
        .fetch_optional(&self.db)
        .await
    }
}

impl FromRequestParts<AppState> for Repository {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Repository {
            db: state.db.clone(),
        })
    }
}

#[cfg(test)]
impl From<PgPool> for Repository {
    fn from(db: PgPool) -> Self {
        Self { db }
    }
}
