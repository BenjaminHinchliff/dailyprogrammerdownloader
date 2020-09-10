use clap::{load_yaml, value_t, App};
use reqwest::header;
use url::Url;

mod authentication;
use authentication::cached_authenticate;

mod commentspage;
mod download;
mod refresh;
mod wikipage;

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

    let id = matches.value_of("id").unwrap();
    let id = if id == "all" {
        (1..String::from_utf8_lossy(&cache.get("challenges-max")?.unwrap()).parse::<i32>()?)
            .collect()
    } else {
        vec![value_t!(matches.value_of("id"), i32)
            .map_err(|_| Error::InvalidArgument("id must be an int or 'all'"))?]
    };
    let difficulties: Vec<_> = matches.values_of("difficulties").unwrap().collect();
    let num_posts = download::download(id, difficulties, &reddit_oauth_base, &client, &cache).await?;
    println!("downloaded {} posts", num_posts);

    cache.flush_async().await?;

    Ok(())
}
