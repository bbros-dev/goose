use crate::calibrate::error;
use async_trait::async_trait;
use hyper::{body::HttpBody as _, body::to_bytes, client::HttpConnector, Body, Client as HyperClient, Method, Request};
use hyper_tls::HttpsConnector;
use lazy_static::lazy_static;
use serde::Deserialize;
use serde_json::from_slice;
use tokio::io::AsyncWriteExt;

// A simple type alias so as to DRY.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

const URI: &str = "https://127.0.0.1";

// lazy_static::lazy_static! {
//     static ref SERVER: tokio::sync::RwLock<Server> = tokio::sync::RwLock::new(Server::new());
// }

#[async_trait]
pub trait HttpClient: Send + Sync + Clone + 'static {
    async fn get_cat_fact(&self) -> Result<String>;
}

#[derive(Clone)]
pub struct Client {
    pub client: HyperClient<HttpsConnector<HttpConnector>>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            client: https_client(),
        }
    }

    fn get_url(&self) -> hyper::Uri {
        // HTTPS requires picking a TLS implementation, so give a better
        // warning if the user tries to request an 'https' URL.
        //let url =
        URI.to_owned().parse::<hyper::Uri>().unwrap()
        // if url.scheme_str() != Some("http") {
        //     println!("This example only works with 'http' URLs.");
        //     return "http://127.0.0.1".parse::<hyper::Uri>().unwrap();
        // }
        //url
    }
}

fn https_client() -> HyperClient<hyper_tls::HttpsConnector<hyper::client::HttpConnector>> {
    let https = hyper_tls::HttpsConnector::new();
    HyperClient::builder().build::<_, Body>(https)
}

// fn with_http_client(
//     http_client: impl HttpClient,
// ) -> impl Filter<Extract = (impl HttpClient,), Error = Infallible> + Clone {
//     warp::any().map(move || http_client.clone())
// }

async fn fetch_url(url: hyper::Uri) -> Result<()> {
    let client = Client::new().client;

    let mut res = client.get(url).await?;

    println!("Response: {}", res.status());
    println!("Headers: {:#?}\n", res.headers());

    // Stream the body, writing each chunk to stdout as we get it
    // (instead of buffering and printing at the end).
    while let Some(next) = res.data().await {
        let chunk = next?;
        tokio::io::stdout().write_all(&chunk).await?;
    }

    println!("\n\nDone!");

    Ok(())
}

// #[async_trait]
// impl HttpClient for Client {
//     async fn get_cat_fact(&self) -> Result<String> {
//         let req = Request::builder()
//             .method(Method::GET)
//             .uri(&format!("{}{}", self.get_url(), "/facts/random"))
//             .header("content-type", "application/json")
//             .header("accept", "application/json")
//             .body(Body::empty())?;
//         let res = self.client.request(req).await?;
//         if !res.status().is_success() {
//             return Err(error::Error::GetCatFactError(res.status()));
//         }
//         let body_bytes = to_bytes(res.into_body()).await?;
//         let json = from_slice::<CatFact>(&body_bytes)?;
//         Ok(json.text)
//     }
// }
