use argon2::Argon2;
use argon2::Params;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use secrecy::ExposeSecret;
use secrecy::SecretString;
use sqlx::types::Uuid;

use crate::Repository;

#[derive(Debug, thiserror::Error)]
pub enum SignUpError {
    #[error("Username already exists")]
    UsernameExists,
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

pub trait SignUpRepository {
    /// returns user_id which can be used to instantiate an AuthenticatedUser
    fn add_new_user(
        &self,
        username: &str,
        password: SecretString,
    ) -> impl std::future::Future<Output = Result<String, SignUpError>> + Send;
}

impl SignUpRepository for Repository {
    #[tracing::instrument(name = "Add new admin user.", skip(self, password))]
    async fn add_new_user(
        &self,
        username: &str,
        password: SecretString,
    ) -> Result<String, SignUpError> {
        use argon2::PasswordHasher;

        let user_id = Uuid::new_v4().to_string();

        let salt = SaltString::generate(&mut OsRng);
        let pw_hash = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            Params::new(15000, 2, 1, None).unwrap(),
        )
        .hash_password(password.expose_secret().as_bytes(), &salt)
        .unwrap()
        .to_string();

        match sqlx::query!(
            r#"
            INSERT INTO users (user_id, username, password_hash)
            VALUES ($1, $2, $3)
            "#,
            user_id,
            username,
            pw_hash,
        )
        .execute(&self.0)
        .await
        {
            Ok(_) => Ok(user_id),
            Err(sqlx::Error::Database(db_err)) => {
                // SQLite unique constraint violation error code is 2067
                if db_err.code().as_deref() == Some("2067") {
                    Err(SignUpError::UsernameExists)
                } else {
                    Err(SignUpError::DatabaseError(sqlx::Error::Database(db_err)))
                }
            }
            Err(e) => Err(SignUpError::DatabaseError(e)),
        }
    }
}
