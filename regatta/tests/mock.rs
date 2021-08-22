//use crate::db::{DBAccessor, Todo};
//use crate::calibrate::error::*;
//use crate::client::HttpClient;
//use crate::calibrate::{error, run};
//use mobc::async_trait;
// use std::sync::atomic::{AtomicBool, Ordering};
// use std::thread;
// use std::time::Duration;
// use wiremock::{
//     matchers::{method, path},
//     Mock, MockServer, ResponseTemplate,
// };

// #[derive(Clone)]
// pub struct MockHttpClient {}
// type Result<T> = std::result::Result<T, error::Error>;

// #[async_trait]
// impl HttpClient for MockHttpClient {
//     async fn get_cat_fact(&self) -> Result<String> {
//         Ok(String::from("cat fact"))
//     }
// }

// #[derive(Clone)]
// pub struct WiremockServer {
//     pub server: Option<MockServer>,
// }

// impl WiremockServer {
//     pub fn new() -> Self {
//         Self { server: None }
//     }

//     pub async fn init(&mut self) {
//         let mock_server = MockServer::start().await;
//         Mock::given(method("GET"))
//             .and(path("/facts/random"))
//             .respond_with(
//                 ResponseTemplate::new(200).set_body_string(r#"{"text": "wiremock cat fact"}"#),
//             )
//             .mount(&mock_server)
//             .await;
//         self.server = Some(mock_server);
//     }
// }
