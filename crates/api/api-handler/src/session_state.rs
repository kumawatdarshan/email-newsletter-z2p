use axum::{extract::FromRequestParts, http::request::Parts};
use tower_sessions::Session;
use tower_sessions::session::Error;
pub struct TypedSession(Session);

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Failed to Insert {0} into session")]
    InsertionError(#[source] Error),
    #[error("Failed to Fetch {0} from session")]
    FetchError(#[source] Error),
    #[error("Failed to cycle session id")]
    CycleError(#[source] Error),
}

type Result<T> = core::result::Result<T, SessionError>;

impl TypedSession {
    const USER_ID_KEY: &'static str = "user_id";
    const USERNAME_KEY: &'static str = "username";

    pub async fn cycle_id(&self) -> Result<()> {
        self.0.cycle_id().await.map_err(SessionError::CycleError)
    }

    pub async fn get_user_id(&self) -> Result<Option<String>> {
        self.0
            .get(Self::USER_ID_KEY)
            .await
            .map_err(SessionError::FetchError)
    }

    pub async fn insert_user_id(&self, user_id: &str) -> Result<()> {
        self.0
            .insert(Self::USER_ID_KEY, &user_id)
            .await
            .map_err(SessionError::InsertionError)
    }

    pub async fn get_username(&self) -> Result<Option<String>> {
        self.0
            .get(Self::USERNAME_KEY)
            .await
            .map_err(SessionError::FetchError)
    }

    pub async fn insert_username(&self, username: &str) -> Result<()> {
        self.0
            .insert(Self::USERNAME_KEY, &username)
            .await
            .map_err(SessionError::InsertionError)
    }
}

impl<S> FromRequestParts<S> for TypedSession
where
    S: Send + Sync,
{
    type Rejection = <Session as FromRequestParts<S>>::Rejection;

    async fn from_request_parts(
        req: &mut Parts,
        state: &S,
    ) -> core::result::Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(req, state).await?;

        Ok(Self(session))
    }
}

// since this is just a convenience wrapper, its not in impl
pub async fn save_session(session: &TypedSession, user_id: &str, username: &str) -> Result<()> {
    session.insert_user_id(user_id).await?;
    session.insert_username(username).await?;

    Ok(())
}
