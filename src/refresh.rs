use chrono::prelude::*;
use regex::Regex;
use scraper::{Html, Selector};
use url::Url;

use super::wikipage::WikiPage;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to decode entities")]
    EntityDecodeError(htmlescape::DecodeErr),
    #[error("{0} in the document is none")]
    HtmlNoneError(&'static str),
    #[error("{0} attribute is missing")]
    NoAttr(&'static str),
}

pub async fn refresh(
    base: &Url,
    client: &reqwest::Client,
    cache: &mut sled::Db,
) -> Result<(), anyhow::Error> {
    println!("refreshing challenges list...");

    let res = client
        .get(base.join("/r/dailyprogrammer/wiki/challenges")?)
        .send()
        .await?;
    let json = res.json::<WikiPage>().await?;

    let corrected_content = htmlescape::decode_html(&json.data.content_html)
        .map_err(|e| Error::EntityDecodeError(e))?
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
            .ok_or_else(|| Error::HtmlNoneError("content element"))?
            .next_sibling()
            .ok_or_else(|| Error::HtmlNoneError("table element"))?,
    )
    .ok_or_else(|| Error::HtmlNoneError("conversion error"))?;

    let trs = challenges.select(&tr_selector);

    let not_int_regex = Regex::new("[^0-9]")?;
    let mut max = 0;
    for tr in trs {
        let mut tds = tr.select(&td_selector);
        let id = not_int_regex
            .replace_all(
                &tds.nth(1)
                    .ok_or_else(|| Error::HtmlNoneError("id"))?
                    .inner_html(),
                "",
            )
            .parse::<i32>()?;
        max = std::cmp::max(max, id);
        let difficulty = tds
            .next()
            .ok_or_else(|| Error::HtmlNoneError("difficulty"))?
            .inner_html();
        let href = tr
            .select(&a_selector)
            .last()
            .ok_or_else(|| Error::HtmlNoneError("link"))?
            .value()
            .attr("href")
            .ok_or_else(|| Error::NoAttr("href"))?;
        let href = Url::parse(href)?;
        cache.insert(
            format!("challenge-{}-{}", id, difficulty.to_lowercase()),
            href.path(),
        )?;
    }

    cache.insert("challenges-max", max.to_string().as_bytes())?;
    cache.insert("challenges", Utc::now().to_rfc3339().as_bytes())?;

    println!("refreshed the challenges!");
    Ok(())
}
