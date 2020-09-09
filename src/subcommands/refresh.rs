use anyhow::Error;
use url::Url;
use scraper::{Html, Selector};

use super::wikipage::WikiPage;

pub async fn refresh(
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