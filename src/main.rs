use z2p::{listener, routes};
pub const PORT: u16 = 3000;
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let listener = listener(PORT).await;
    let app = routes();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
