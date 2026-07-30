use crate::types::gentoo::AtomFeed;

use std::time::Duration;

use chrono::{ DateTime, FixedOffset };

use reqwest::Client;

use twilight_gateway::MessageSender;

use twilight_model::gateway::presence::{
  Activity,
  ActivityType,
  MinimalActivity,
  Status
};

use twilight_model::gateway::payload::outgoing::UpdatePresence;

const ADDED_FEED:   &str = "https://packages.gentoo.org/packages/added.atom";
const UPDATED_FEED: &str = "https://packages.gentoo.org/packages/updated.atom";

const REFRESH_INTERVAL: Duration = Duration::from_secs(15 * 60);

async fn latest_entry(
  client: &Client,
  url: &str
) -> anyhow::Result<(String, DateTime<FixedOffset>)> {
  let body = client.get(url).send().await?.text().await?;
  let feed: AtomFeed = quick_xml::de::from_str(&body)?;
  let entry = feed.entries.into_iter().next()
    .ok_or_else(|| anyhow!("empty atom feed: {url}"))?;
  let updated = DateTime::parse_from_rfc3339(&entry.updated)?;
  Ok((entry.title, updated))
}

async fn latest_package_title(client: &Client) -> anyhow::Result<String> {
  let (added_title, added_at)     = latest_entry(client, ADDED_FEED).await?;
  let (updated_title, updated_at) = latest_entry(client, UPDATED_FEED).await?;
  Ok(if updated_at > added_at { updated_title } else { added_title })
}

async fn refresh_presence(sender: &MessageSender, client: &Client) -> anyhow::Result<()> {
  let title = latest_package_title(client).await?;
  let activity: Activity = MinimalActivity {
    kind: ActivityType::Watching,
    name: title,
    url: None
  }.into();
  let presence = UpdatePresence::new([activity], false, None, Status::Online)?;
  sender.command(&presence)?;
  Ok(())
}

pub fn spawn_presence_updater(sender: MessageSender, client: Client) {
  tokio::spawn(async move {
    loop {
      if let Err(why) = refresh_presence(&sender, &client).await {
        tracing::warn!("failed to update presence from Gentoo package feeds: {why:?}");
      }
      tokio::time::sleep(REFRESH_INTERVAL).await;
    }
  });
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_added_feed_fixture() {
    let xml = include_str!("../tests/fixtures/added.atom");
    let feed: AtomFeed = quick_xml::de::from_str(xml).unwrap();
    let first = feed.entries.first().unwrap();
    assert_eq!(first.title, "games-fps/ut2004-bonuspack-xp");
    DateTime::parse_from_rfc3339(&first.updated).unwrap();
  }

  #[test]
  fn parses_updated_feed_fixture() {
    let xml = include_str!("../tests/fixtures/updated.atom");
    let feed: AtomFeed = quick_xml::de::from_str(xml).unwrap();
    let first = feed.entries.first().unwrap();
    assert_eq!(first.title, "sys-kernel/gentoo-sources-6.18.41");
    DateTime::parse_from_rfc3339(&first.updated).unwrap();
  }

  #[tokio::test]
  #[ignore = "hits the real packages.gentoo.org feeds"]
  async fn fetches_latest_package_title_from_live_feeds() {
    let client = Client::new();
    let title = latest_package_title(&client).await.unwrap();
    assert!(!title.is_empty());
  }
}
