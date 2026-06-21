use password_auth::VerifyError;
use crate::{error::AppError, repository::Repository, models::UserRecord};

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
                sqlx::Error::Database(db_err) if db_err.is_unique_violation() => AppError::UsernameTaken,
                err => AppError::Database(err),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn test_register_and_authenticate(pool: PgPool) {
        let repo = Repository::from(pool);
        let unauth = UnauthenticatedUser::new("testuser".to_string(), "senha_segura123".to_string());
        
        // Testa o registro (consumindo uma nova instância temporária, já que register pega `self`)
        let user = UnauthenticatedUser::new("testuser".to_string(), "senha_segura123".to_string())
            .register(&repo)
            .await
            .expect("Registro falhou");
        assert_eq!(user.username, "testuser");

        // Testa autenticação correta
        let auth_result = unauth.authenticate(&repo).await.expect("Autenticacao falhou");
        assert_eq!(auth_result.username, "testuser");

        // Testa autenticação com senha errada
        let bad_unauth = UnauthenticatedUser::new("testuser".to_string(), "senha_errada".to_string());
        let bad_auth_result = bad_unauth.authenticate(&repo).await;
        assert!(matches!(bad_auth_result, Err(AppError::InvalidCredentials)));
    }
}
