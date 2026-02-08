use configuration::RedisConfiguration;
use secrecy::ExposeSecret;

/// This crate needs refactor in some point of time.
// TODO:do-me
pub async fn create_redis_pool(
    config: &RedisConfiguration,
) -> anyhow::Result<tower_sessions_redis_store::fred::prelude::Pool> {
    use tower_sessions_redis_store::fred::{
        clients::Pool, interfaces::ClientLike, prelude::ServerConfig, types::config::Config,
    };

    let config = Config {
        server: ServerConfig::new_centralized(config.host.expose_secret(), config.port),
        ..Config::default()
    };

    let redis_pool = Pool::new(config, None, None, None, 6)?;

    let _redis_join_handle = redis_pool.connect();

    redis_pool.wait_for_connect().await?;

    Ok(redis_pool)
}
