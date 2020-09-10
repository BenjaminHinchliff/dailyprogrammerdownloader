use tokio::fs;
use tokio::prelude::*;

use super::commentspage::Comment;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("challenge doesn't exist")]
    InvalidChallenge,
}

pub async fn get(
    id: i32,
    difficulties: &Vec<&str>,
    base: &url::Url,
    client: &reqwest::Client,
    cache: &sled::Db,
) -> Result<(), anyhow::Error> {
    let challenge_key = format!("challenge-{}-{}", id, difficulties[0]);
    let url = base.join(&String::from_utf8_lossy(
        &cache
            .get(&challenge_key)?
            .ok_or_else(|| Error::InvalidChallenge)?,
    ))?;

    let comments = client.get(url).send().await?.json::<Vec<Comment>>().await?;
    let post = &comments[0].data.children[0].data;
    let title = &post.fields["title"];
    let text = &post.fields["selftext"];
    println!("{}", text);
    let mut file = fs::File::create(challenge_key + ".md").await?;
    file.write_all(b"# ").await?;
    file.write_all(
        title
            .as_str()
            .ok_or_else(|| Error::InvalidChallenge)?
            .as_bytes(),
    )
    .await?;
    file.write_all(b"\n\n").await?;
    file.write_all(
        text.as_str()
            .ok_or_else(|| Error::InvalidChallenge)?
            .as_bytes(),
    )
    .await?;

    Ok(())
}
