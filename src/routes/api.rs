use crate::{
    app::AppState, auth::user::User, error::AppError, models::Asset, repository::Repository,
};
use axum::{extract::Path, routing::get, Json, Router};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/assets", get(list_assets).post(create_asset))
        .route(
            "/assets/{id}",
            axum::routing::patch(update_asset).delete(delete_asset),
        )
}

async fn list_assets(user: User, repository: Repository) -> Result<Json<Vec<Asset>>, AppError> {
    Ok(Json(repository.list_assets_by_user(user.id()).await?))
}

#[derive(Deserialize)]
struct CreateAssetRequest {
    name: String,
    quantity: f64,
    unit_value: f64,
}

async fn create_asset(
    user: User,
    repository: Repository,
    Json(req): Json<CreateAssetRequest>,
) -> Result<Json<Asset>, AppError> {
    if req.quantity <= 0.0 || req.unit_value <= 0.0 {
        return Err(AppError::InvalidAssetData);
    }
    Ok(Json(
        repository
            .create_asset(user.id(), req.name, req.quantity, req.unit_value)
            .await?,
    ))
}

#[derive(Deserialize)]
struct UpdateAssetRequest {
    name: Option<String>,
    quantity: Option<f64>,
    unit_value: Option<f64>,
}

async fn update_asset(
    user: User,
    repository: Repository,
    Path(id): Path<i64>,
    Json(req): Json<UpdateAssetRequest>,
) -> Result<Json<Asset>, AppError> {
    repository
        .update_asset(id, user.id(), req.name, req.quantity, req.unit_value)
        .await?
        .ok_or(AppError::AssetDoesNotExist)
        .map(Json)
}

async fn delete_asset(
    user: User,
    repository: Repository,
    Path(id): Path<i64>,
) -> Result<axum::http::StatusCode, AppError> {
    let rows = repository.delete_asset(id, user.id()).await?;
    if rows == 0 {
        return Err(AppError::AssetDoesNotExist);
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    async fn setup_user(pool: &PgPool, username: &str) -> User {
        let repo = Repository::from(pool.clone());
        let record = repo.add_user(username, "hash").await.unwrap();
        User::new(record.id, record.username)
    }

    #[sqlx::test]
    async fn test_create_valid_asset(pool: PgPool) {
        let user = setup_user(&pool, "user1").await;
        let req = Json(CreateAssetRequest {
            name: "PETR4".into(),
            quantity: 10.0,
            unit_value: 35.0,
        });
        let result = create_asset(
            User::new(user.id(), user.username().to_string()),
            Repository::from(pool),
            req,
        )
        .await
        .unwrap();

        assert_eq!(result.0.name, "PETR4");
    }

    #[sqlx::test]
    async fn test_create_invalid_asset(pool: PgPool) {
        let user = setup_user(&pool, "user1").await;
        let req = Json(CreateAssetRequest {
            name: "PETR4".into(),
            quantity: -10.0,
            unit_value: 35.0,
        });
        let err = create_asset(
            User::new(user.id(), user.username().to_string()),
            Repository::from(pool),
            req,
        )
        .await
        .unwrap_err();

        assert!(matches!(err, AppError::InvalidAssetData));
    }

    #[sqlx::test]
    async fn test_asset_isolation_and_delete(pool: PgPool) {
        let user1 = setup_user(&pool, "user1").await;
        let user2 = setup_user(&pool, "user2").await;

        let repo = Repository::from(pool.clone());
        let asset1 = repo
            .create_asset(user1.id(), "VALE3".into(), 10.0, 60.0)
            .await
            .unwrap();
        repo.create_asset(user2.id(), "WEGE3".into(), 5.0, 40.0)
            .await
            .unwrap();

        // test isolation on list
        let result1 = list_assets(
            User::new(user1.id(), "".into()),
            Repository::from(pool.clone()),
        )
        .await
        .unwrap();
        assert_eq!(result1.0.len(), 1);
        assert_eq!(result1.0[0].name, "VALE3");

        let result2 = list_assets(
            User::new(user2.id(), "".into()),
            Repository::from(pool.clone()),
        )
        .await
        .unwrap();
        assert_eq!(result2.0.len(), 1);
        assert_eq!(result2.0[0].name, "WEGE3");

        // test isolation on update
        let req_update = Json(UpdateAssetRequest {
            name: Some("MUDOU".into()),
            quantity: None,
            unit_value: None,
        });
        let update_err = update_asset(
            User::new(user2.id(), "".into()),
            Repository::from(pool.clone()),
            Path(asset1.id),
            req_update,
        )
        .await
        .unwrap_err();
        assert!(matches!(update_err, AppError::AssetDoesNotExist));

        // test isolation on delete nonexistent
        let delete_err = delete_asset(
            User::new(user1.id(), "".into()),
            Repository::from(pool.clone()),
            Path(999),
        )
        .await
        .unwrap_err();
        assert!(matches!(delete_err, AppError::AssetDoesNotExist));

        // test valid delete
        let delete_ok = delete_asset(
            User::new(user1.id(), "".into()),
            Repository::from(pool.clone()),
            Path(asset1.id),
        )
        .await
        .unwrap();
        assert_eq!(delete_ok, axum::http::StatusCode::NO_CONTENT);
    }

    #[sqlx::test(fixtures("fixtures/sample_assets.sql"))]
    async fn test_crud_flow_with_fixture(pool: PgPool) {
        // the fixture inserts user 1000 and 3 assets
        let user = User::new(1000, "brucewayne".into());

        // 1. List assets (should be 3)
        let assets = list_assets(
            User::new(user.id(), "".into()),
            Repository::from(pool.clone()),
        )
        .await
        .unwrap()
        .0;
        assert_eq!(assets.len(), 3);

        // 2. Edit an asset (asset 1001 is WAYN3)
        let update_req = Json(UpdateAssetRequest {
            name: None,
            quantity: Some(1500.0),
            unit_value: None,
        });
        let updated = update_asset(
            User::new(user.id(), "".into()),
            Repository::from(pool.clone()),
            Path(1001),
            update_req,
        )
        .await
        .unwrap()
        .0;
        assert_eq!(updated.quantity, 1500.0);

        // 3. Delete an asset (asset 1002 is BTC)
        let delete_ok = delete_asset(
            User::new(user.id(), "".into()),
            Repository::from(pool.clone()),
            Path(1002),
        )
        .await
        .unwrap();
        assert_eq!(delete_ok, axum::http::StatusCode::NO_CONTENT);

        // 4. List again (should be 2)
        let remaining = list_assets(
            User::new(user.id(), "".into()),
            Repository::from(pool.clone()),
        )
        .await
        .unwrap()
        .0;
        assert_eq!(remaining.len(), 2);
    }
}
