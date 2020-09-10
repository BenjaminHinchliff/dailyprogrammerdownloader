use clap::{load_yaml, value_t, App};
use reqwest::header;
use url::Url;

mod authentication;
use authentication::cached_authenticate;

mod commentspage;
mod wikipage;
mod refresh;
mod download;

const REDDIT_OAUTH_BASE: &'static str = "https://oauth.reddit.com";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid argument: {0}")]
    InvalidArgument(&'static str),
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let yaml = load_yaml!("cli.yml");
    let app = App::from_yaml(yaml);
    let matches = app.get_matches();

    let mut cache = sled::open("./cache")?;

    let token = cached_authenticate(&mut cache).await?;

    let reddit_oauth_base = Url::parse(REDDIT_OAUTH_BASE)?;

    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&("bearer ".to_owned() + &token)).unwrap(),
    );

    let client = reqwest::Client::builder()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION"),
        ))
        .default_headers(headers)
        .build()?;

    if cache.get("challenges").unwrap().is_none() || matches.is_present("refresh") {
        refresh::refresh(&reddit_oauth_base, &client, &mut cache).await?;
    }

    let id = value_t!(matches.value_of("id"), i32)
        .map_err(|_| Error::InvalidArgument("id must be an int"))?;
    let difficulties: Vec<_> = matches.values_of("difficulties").unwrap().collect();
    download::download(id, &difficulties, &reddit_oauth_base, &client, &cache).await?;

    cache.flush_async().await?;

    Ok(())
}
