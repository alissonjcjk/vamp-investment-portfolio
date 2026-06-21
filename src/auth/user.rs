use axum::extract::FromRequestParts;
use axum_extra::extract::CookieJar;
use jwt_simple::prelude::*;
use password_auth::VerifyError;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

use crate::{app::AppState, error::AppError, models::UserRecord, repository::Repository};

const JWT_DURATION_MINUTES: u64 = 60 * 24; // 24h de sessão

pub struct UnauthenticatedUser {
    username: String,
    password: String,
}

impl UnauthenticatedUser {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    pub async fn authenticate(&self, repository: &Repository) -> Result<UserRecord, AppError> {
        let record = repository
            .get_user_by_name(&self.username)
            .await?
            .ok_or(AppError::UserDoesNotExist)?;

        match password_auth::verify_password(&self.password, &record.password_hash) {
            Ok(()) => Ok(record),
            Err(VerifyError::PasswordInvalid) => Err(AppError::InvalidCredentials),
            Err(VerifyError::Parse(err)) => panic!("Falha no algoritmo de hash: {err}"),
        }
    }

    pub async fn register(self, repository: &Repository) -> Result<UserRecord, AppError> {
        let password_hash = password_auth::generate_hash(self.password);
        repository
            .add_user(&self.username, &password_hash)
            .await
            .map_err(|err| match err {
                sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                    AppError::UsernameTaken
                }
                err => AppError::Database(err),
            })
    }
}

pub struct User {
    id: i64,
    username: String,
}

impl User {
    pub fn new(id: i64, username: String) -> Self {
        Self { id, username }
    }

    pub const fn id(&self) -> i64 {
        self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn auth_token(&self) -> Result<String, AppError> {
        let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET deve estar definida no .env");
        let key = HS256Key::from_bytes(secret.as_bytes());
        let claims = Claims::with_custom_claims(
            UserClaims {
                id: self.id,
                username: self.username.clone(),
            },
            Duration::from_mins(JWT_DURATION_MINUTES),
        );
        Ok(key.authenticate(claims)?)
    }

    pub fn from_auth_token(token: &str) -> Result<Self, AppError> {
        let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET deve estar definida no .env");
        let key = HS256Key::from_bytes(secret.as_bytes());
        let options = VerificationOptions {
            time_tolerance: Some(Duration::from_secs(0)),
            ..Default::default()
        };
        let claims: UserClaims = key.verify_token::<UserClaims>(token, Some(options))?.custom;
        Ok(Self::new(claims.id, claims.username))
    }
}

#[derive(Serialize, Deserialize)]
struct UserClaims {
    id: i64,
    username: String,
}

impl FromRequestParts<AppState> for User {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let token = jar.get("token").ok_or(AppError::MissingAuthorization)?;
        User::from_auth_token(token.value())
    }
}

impl FromRequestParts<AppState> for Option<User> {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(User::from_request_parts(parts, state).await.ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn test_register_and_authenticate(pool: PgPool) {
        let repo = Repository::from(pool);
        let unauth =
            UnauthenticatedUser::new("testuser".to_string(), "senha_segura123".to_string());

        let user = UnauthenticatedUser::new("testuser".to_string(), "senha_segura123".to_string())
            .register(&repo)
            .await
            .expect("Registro falhou");
        assert_eq!(user.username, "testuser");

        let auth_result = unauth
            .authenticate(&repo)
            .await
            .expect("Autenticacao falhou");
        assert_eq!(auth_result.username, "testuser");

        let bad_unauth =
            UnauthenticatedUser::new("testuser".to_string(), "senha_errada".to_string());
        let bad_auth_result = bad_unauth.authenticate(&repo).await;
        assert!(matches!(bad_auth_result, Err(AppError::InvalidCredentials)));
    }

    #[sqlx::test]
    async fn test_register_duplicate(pool: PgPool) {
        let repo = Repository::from(pool);
        UnauthenticatedUser::new("alice".to_string(), "senha".to_string())
            .register(&repo)
            .await
            .unwrap();

        let duplicate = UnauthenticatedUser::new("alice".to_string(), "123".to_string())
            .register(&repo)
            .await
            .unwrap_err();

        assert!(matches!(duplicate, AppError::UsernameTaken));
    }

    #[sqlx::test]
    async fn test_authenticate_invalid_user(pool: PgPool) {
        let repo = Repository::from(pool);
        let unauth = UnauthenticatedUser::new("fantasma".to_string(), "123".to_string());
        let err = unauth.authenticate(&repo).await.unwrap_err();
        assert!(matches!(err, AppError::UserDoesNotExist));
    }

    #[test]
    fn test_jwt_generation_and_validation() {
        std::env::set_var("JWT_SECRET", "test-secret-key");

        let user = User::new(1, "tester".to_string());
        let token = user.auth_token().expect("Failed to generate token");

        let decoded_user = User::from_auth_token(&token).expect("Failed to decode token");
        assert_eq!(decoded_user.id(), 1);
        assert_eq!(decoded_user.username(), "tester");
    }

    #[test]
    fn test_jwt_expiration() {
        std::env::set_var("JWT_SECRET", "test-secret-key");

        let user = User::new(2, "expired_tester".to_string());
        let secret = "test-secret-key";
        let key = HS256Key::from_bytes(secret.as_bytes());

        // Simulating expired token by setting custom expiration time
        let claims = Claims::with_custom_claims(
            UserClaims {
                id: user.id,
                username: user.username.clone(),
            },
            Duration::from_secs(1), // Expires in slightly more than 0 to allow sleep
        );

        let token = key.authenticate(claims).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(1500));

        // Validation should fail
        let decode_result = User::from_auth_token(&token);
        assert!(
            decode_result.is_err(),
            "Decoding should fail for expired token"
        );
    }
}
