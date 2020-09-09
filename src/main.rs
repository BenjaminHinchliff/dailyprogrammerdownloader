use anyhow::Error;
use reqwest::header;
use scraper::{Html, Selector};
use url::Url;

mod authentication;
use authentication::cached_authenticate;

mod wikipage;
use wikipage::WikiPage;

const REDDIT_OAUTH_BASE: &'static str = "https://oauth.reddit.com";

pub async fn refresh_challenges(
    base: &Url,
    client: &reqwest::Client,
    cache: &mut sled::Db,
) -> Result<(), Error> {
    let res = client
        .get(base.join("/r/dailyprogrammer/wiki/challenges").unwrap())
        .send()
        .await
        .expect("failed to fetch challenges index");
    let json = res
        .json::<WikiPage>()
        .await
        .expect("failed to get challenge index response");

    let corrected_content = htmlescape::decode_html(&json.data.content_html)
        .unwrap()
        .replace("\n", "");

    let content = Html::parse_fragment(&corrected_content);
    let challenges_selector = Selector::parse("#wiki_challenge").unwrap();
    let tr_selector = Selector::parse("tbody tr").unwrap();
    let td_selector = Selector::parse("td").unwrap();
    let a_selector = Selector::parse("a").unwrap();

    let challenges = scraper::ElementRef::wrap(
        content
            .select(&challenges_selector)
            .next()
            .expect("failed to select challenge title")
            .next_sibling()
            .expect("failed to get challenge table"),
    )
    .expect("failed to convert table node to element");

    let trs = challenges.select(&tr_selector);

    for tr in trs {
        let mut tds = tr.select(&td_selector);
        let id = tds
            .nth(1)
            .expect("failed to get id for a post")
            .inner_html();
        let difficulty = tds
            .next()
            .expect("failed to get a difficulty for a post")
            .inner_html();
        let post = tr
            .select(&a_selector)
            .last()
            .expect("failed to get the link for a post")
            .value()
            .attr("href")
            .expect("failed to extract href from post link");
        cache
            .insert(
                format!("challenge-{}-{}", id, difficulty.to_lowercase()),
                post,
            )
            .expect("failed to insert a post into the cache");
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let mut cache = sled::open("./cache").expect("failed to open cache");

    let token = cached_authenticate(&mut cache)
        .await
        .expect("failed to get token");

    println!("{}", token);

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

    refresh_challenges(&reddit_oauth_base, &client, &mut cache)
        .await
        .expect("failed to refresh challenges");

    cache
        .flush_async()
        .await
        .expect("failed to flush sled data");
}
