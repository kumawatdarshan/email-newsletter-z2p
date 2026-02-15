use axum::{extract::FromRequestParts, http::request::Parts};
use tower_sessions::Session;

type Result<T> = core::result::Result<T, tower_sessions::session::Error>;

pub struct TypedSession(Session);

impl TypedSession {
    const USER_ID_KEY: &'static str = "user_id";
    const USERNAME_KEY: &'static str = "username";

    pub async fn cycle_id(&self) -> Result<()> {
        self.0.cycle_id().await
    }

    pub async fn get_user_id(&self) -> Result<Option<String>> {
        self.0.get(Self::USER_ID_KEY).await
    }

    pub async fn insert_user_id(
        &self,
        user_id: &str,
    ) -> std::result::Result<(), tower_sessions::session::Error> {
        self.0.insert(Self::USER_ID_KEY, &user_id).await
    }

    pub async fn get_username(&self) -> Result<Option<String>> {
        self.0.get(Self::USERNAME_KEY).await
    }

    pub async fn insert_username(
        &self,
        username: &str,
    ) -> std::result::Result<(), tower_sessions::session::Error> {
        self.0.insert(Self::USERNAME_KEY, &username).await
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
