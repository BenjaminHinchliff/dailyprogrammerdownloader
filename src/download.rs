use itertools::Itertools;
use tokio::fs;
use tokio::prelude::*; // write_all
use futures::future::join_all;

use crate::commentspage::Comment;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("challenge doesn't exist")]
    InvalidChallenge,
}

pub async fn download<'a, T, U>(
    id: T,
    diffs: U,
    base: &url::Url,
    client: &reqwest::Client,
    cache: &sled::Db,
) -> Result<usize, anyhow::Error>
where
    T: IntoIterator<Item = i32>,
    T::Item: Clone,
    U: IntoIterator<Item = &'a str>,
    U::IntoIter: Clone,
{
    let tasks = id.into_iter().cartesian_product(diffs.into_iter()).map(|(id, diff)| {
        async move {
            let challenge_key = format!("challenge-{}-{}", id, diff);
            let url = base.join(&String::from_utf8_lossy(
                &cache
                    .get(&challenge_key)?
                    .ok_or_else(|| Error::InvalidChallenge)?,
            ))?;

            let comments = client.get(url).send().await?.json::<Vec<Comment>>().await?;
            let post = &comments[0].data.children[0].data;
            let title = &post.fields["title"];
            let text = &post.fields["selftext"];
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
            Ok::<(), anyhow::Error>(())
        }
    });

    // complete tasks and return number of successes
    Ok(join_all(tasks).await.into_iter().filter_map(Result::ok).count())
}
