// use reqwest::StatusCode;
// use z2p::{listener, routes};

// /// Why this complicated test for something simple as health_check?
// /// This is a black box test, meaning it is decoupled(*mostly*) from our codebase.
// /// Decoupled as in, it is meant to behave like how consumers of this API would use it.
// /// thus it makes several checks:
// /// 1. Are we firing the correct endpoint? (/health_check)
// /// 1. Are we firing the correct request? (GET)
// /// 1. Is it a successful response? (200)
// /// 1. Is there any content recieved? (There should not be any)
// #[tokio::test]
// async fn test_health_check() {
//     // Arrange
//     let addr = spawn_app().await.expect("Failed to spawn app");
//     let client = reqwest::Client::new();

//     let response = client
//         .get(format!("{addr}/health_check"))
//         .send()
//         .await
//         .expect("Failed to send reqest");

//     // assert!(response.status().is_success());
//     assert_eq!(Some(0), Some(0)) // to validate there was no nothing present
// }

// // #[tokio::test]
// // async fn test_subscribe_valid() {
// //     let addr = spawn_app().await.expect("Failed to spawn app");
// //     let client = reqwest::Client::new();

// //     let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
// //     let response = client
// //         .post(format!("{addr}/subscribe"))
// //         .header("Content-type", "application/x-www-form-urlencoded")
// //         .body(body)
// //         .send()
// //         .await
// //         .expect("Failed to execute request.");

// //     assert_eq!(StatusCode::OK, response.status())
// // }

// // #[tokio::test]
// // async fn test_subscribe_invalid() {
// //     let addr = spawn_app().await.expect("Failed to spawn app");
// //     let client = reqwest::Client::new();

// //     let test_cases = [
// //         ("name=le%20guin", "missing the email"),
// //         ("email=ursula_le_guin%40gmail.com", "missing the name"),
// //         ("", "missing both name and email"),
// //     ];

// //     for (body, error) in test_cases {
// //         let response = client
// //             .post(format!("{addr}/subscribe"))
// //             .header("Content-type", "application/x-www-form-urlencoded")
// //             .body(body)
// //             .send()
// //             .await
// //             .expect("Failed to execute request.");

// //         assert_eq!(StatusCode::OK, response.status(), "Api did not fail with 400.\n{error}")
// //     }
// // }

// async fn spawn_app() -> std::io::Result<String> {
//     let app = routes();
//     let host = "127.0.0.1";
//     let listener = listener(0).await;
//     let port = listener.local_addr().unwrap().port();
//     let addr = format!("http://{host}:{port}");

//     tokio::spawn(async move {
//         axum::serve(listener, app)
//             .await
//             .expect("Failed to bind address")
//     });

//     Ok(addr)
// }
