use clap::{load_yaml, App};
use reqwest::header;
use url::Url;

mod authentication;
use authentication::cached_authenticate;

mod subcommands;

const REDDIT_OAUTH_BASE: &'static str = "https://oauth.reddit.com";

#[tokio::main]
async fn main() {
    let yaml = load_yaml!("cli.yml");
    let app = App::from_yaml(yaml);
    let matches = app.get_matches();
    let (command, _args) = matches.subcommand();

    let mut cache = sled::open("./cache").expect("failed to open cache");

    let token = cached_authenticate(&mut cache)
        .await
        .expect("failed to get token");

    let reddit_oauth_base = Url::parse(REDDIT_OAUTH_BASE).expect("invalid base api url");

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
        .build()
        .expect("failed to build client");

    match command {
        "refresh" => subcommands::refresh(&reddit_oauth_base, &client, &mut cache)
            .await
            .expect("failed to refresh challenges"),
        _ => {
            println!("must specify a subcommand");
        },
    }

    cache
        .flush_async()
        .await
        .expect("failed to flush sled data");
}
