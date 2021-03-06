use std::collections::HashMap;
use std::str::FromStr;

use anyhow::Error;
use chrono::prelude::*;
use tiny_http::{Response, Server, Header};
use url::Url;

const HOUR_AS_SECONDS: i64 = 3600;

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("creation of oauth receive server failed")]
    ServerCreationError,
}

pub async fn authenticate() -> Result<String, Error> {
    let state = rand::random::<u32>();
    let authorize_url = format!(
        "https://www.reddit.com/api/v1/authorize\
            ?client_id=no8ICo67XIf_dQ\
            &response_type=token\
            &state={}\
            &redirect_uri=http://localhost:3000\
            &scope=wikiread read",
        state
    );

    println!("opening oauth2 implicit grant link in default browser...");
    if let Err(_) = webbrowser::open(&authorize_url) {
        println!("failed to open link in browser - open this link in one:");
        println!("{}", authorize_url);
    }

    let server = Server::http("0.0.0.0:3000").map_err(|_| AuthError::ServerCreationError)?;

    let mut response = Response::from_string(include_str!("../public/retrieval.html"));
    response.add_header(Header::from_str("Content-Type: text/html").unwrap());
    let request = server.recv()?;
    request.respond(response)?;
    let request = server.recv()?;
    let url = Url::parse(&("http://localhost:3000".to_owned() + request.url()))?;
    let query: HashMap<_, _> = url.query_pairs().collect();
    assert_eq!(query["scope"], "read wikiread");
    assert_eq!(query["token_type"], "bearer");
    assert_eq!(query["state"], state.to_string());
    request.respond(Response::from_string("token received"))?;

    Ok(query["access_token"].to_string())
}

pub async fn cached_authenticate(cache: &mut sled::Db) -> Result<String, Error> {
    let mut token_expired = false;
    if let Some(token_time_slice) = cache.get("token-time").unwrap() {
        let token_time =
            DateTime::parse_from_rfc3339(&String::from_utf8_lossy(&token_time_slice)).unwrap();
        let current_time = Utc::now();
        let delta = current_time.timestamp() - token_time.timestamp();
        token_expired = delta > HOUR_AS_SECONDS;
    }

    let cache_token = cache.get("token").unwrap();
    Ok(if cache_token.is_none() || token_expired {
        let new_token = authenticate().await.unwrap();
        cache.insert("token", new_token.as_bytes()).unwrap();
        cache
            .insert("token-time", Utc::now().to_rfc3339().as_bytes())
            .unwrap();
        new_token
    } else {
        println!("cached token found!");
        String::from_utf8_lossy(&cache_token.unwrap()).to_string()
    })
}
