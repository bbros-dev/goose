// References:
// - https://github.com/zupzup/rust-web-e2e-testing

//use regatta::calibrate::{client, router};
use hyper::{body::to_bytes, client::HttpConnector, Body, Client as HyperClient, Method, Request};
use hyper_tls::HttpsConnector;
use lazy_static::lazy_static;
use mock::*;
use std::sync::RwLock;
//use warp::test::request;

mod mock;

// lazy_static! {
//     static ref SERVER: tokio::sync::RwLock<Server> = tokio::sync::RwLock::new(Server::new());
//     //pub static ref MOCK_HTTP_SERVER: RwLock<WiremockServer> = RwLock::new(WiremockServer::new());
// }

// Pure Mock Tests with HTTP client and DB mocked
// #[tokio::test]
// async fn test_list_todos_mock() {
//     let r = router(MockHttpClient {});
//     let resp = request().path("/todo").reply(&r).await;
//     assert_eq!(resp.status(), 200);
//     assert_eq!(
//         resp.body(),
//         r#"[{"id":1,"name":"first todo","checked":true}]"#
//     );
// }

// #[tokio::test]
// async fn test_create_todo_mock() {
//     let r = router(MockHttpClient {}, MockDBAccessor {});
//     let resp = request()
//         .path("/todo")
//         .method("POST")
//         .body("")
//         .reply(&r)
//         .await;
//     assert_eq!(resp.status(), 200);
//     assert_eq!(resp.body(), r#"{"id":2,"name":"cat fact","checked":false}"#);
// }

// Hybrid Mock Tests with Mock DB and Wiremock
// #[tokio::test]
// async fn test_create_and_list_todo_hybrid() {
//     setup_wiremock().await;
//     let r = router(http::Client::new(), MockDBAccessor {});
//     let resp = request()
//         .path("/todo")
//         .method("POST")
//         .body("")
//         .reply(&r)
//         .await;
//     assert_eq!(resp.status(), 200);
//     assert_eq!(
//         resp.body(),
//         r#"{"id":2,"name":"wiremock cat fact","checked":false}"#
//     );

//     let resp = request().path("/todo").reply(&r).await;
//     assert_eq!(resp.status(), 200);
//     assert_eq!(
//         resp.body(),
//         r#"[{"id":1,"name":"first todo","checked":true}]"#
//     );
// }

// async fn setup_wiremock() {
//     MOCK_HTTP_SERVER.write().unwrap().init().await;
// }

// Full Test with Wiremock and real DB
// #[tokio::test]
// async fn test_create_and_list_full() {
//     setup_wiremock().await;
//     let r = router(http::Client::new(), init_db().await);
//     let resp = request()
//         .path("/todo")
//         .method("POST")
//         .body("")
//         .reply(&r)
//         .await;
//     assert_eq!(resp.status(), 200);
//     assert_eq!(
//         resp.body(),
//         r#"{"id":1,"name":"wiremock cat fact","checked":false}"#
//     );

//     let resp = request().path("/todo").reply(&r).await;
//     assert_eq!(resp.status(), 200);
//     assert_eq!(
//         resp.body(),
//         r#"[{"id":1,"name":"wiremock cat fact","checked":false}]"#
//     );
// }

// async fn init_db() -> impl db::DBAccessor {
//     let db_pool = db::create_pool().expect("database pool can be created");
//     let db_access = db::DBAccess::new(db_pool.clone());

//     db_access
//         .init_db()
//         .await
//         .expect("database can be initialized");

//     let con = db_pool.get().await.unwrap();
//     let query = format!("BEGIN;DELETE FROM todo;ALTER SEQUENCE todo_id_seq RESTART with 1;COMMIT;");
//     let _ = con.batch_execute(query.as_str()).await;

//     db_access
// }

// E2E Tests with actual server, Wiremock and DB
#[tokio::test]
async fn test_hello_e2e() {
    // setup_wiremock().await;
    regatta::calibrate::init_real_server().await;
    let http_client = regatta::calibrate::client::Client::new().client;

    let req = Request::builder()
        .method(Method::GET)
        .uri("http://localhost:8888/")
        .body(Body::empty())
        .unwrap();
    let (parts, body)  = http_client.request(req).await.unwrap().into_parts();
    //let (parts, body) = resp.into_parts();
    assert_eq!(parts.status, 200);
    let body_bytes = to_bytes(body).await.unwrap();
    assert_eq!(
        body_bytes,
        r#"Hello World!"#
    );

    let req = Request::builder()
        .method(Method::GET)
        .uri("http://localhost:8888")
        .body(Body::empty())
        .unwrap();
    let resp = http_client.request(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(
        body_bytes,
        r#"Hello World!"#
    );
}

#[tokio::test]
async fn test_hello_json_e2e() {
    //setup_wiremock().await;
    regatta::calibrate::init_real_server().await;
    let http_client = regatta::calibrate::client::Client::new().client;

    let req = Request::builder()
        .method(Method::GET)
        .uri("http://localhost:8080/todo")
        .body(Body::empty())
        .unwrap();
    let resp = http_client.request(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body_bytes, r#"[]"#);
}
