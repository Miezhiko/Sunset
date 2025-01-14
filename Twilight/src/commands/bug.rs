use crate::types::{
  common::State,
  gentoo::Bugs
};

use twilight_model::{
  channel::Message,
  util::Timestamp
};

use twilight_util::builder::embed::{
  EmbedBuilder,
  EmbedFieldBuilder,
  EmbedFooterBuilder
};

use chrono::DateTime;

pub async fn bug(msg: Message, number: Option<i32>, state: State) -> anyhow::Result<()> {
  tracing::debug!(
    "bug command in channel {} by {}",
    msg.channel_id,
    msg.author.name
  );
  let mut bug_number = -1;
  match number {
    Some(num) => { bug_number = num },
    None      => {
      for if_num in msg.content.split_whitespace() {
        let try_num = if_num.parse::<i32>();
        match try_num {
          Ok(num) => {
            bug_number = num;
            break;
          }, Err(_) => {
            continue;
          }
        }
      }
    }
  }
  if bug_number < 0 {
    state
      .http
      .create_message(msg.channel_id)
      .reply(msg.id)
      .content("You need to provide bug number to search for")
      .await?;
    return Ok(())
  }
  let res = state
            .request_client
            .get(format!("https://bugs.gentoo.org/rest/bug?id={bug_number}")).send().await?;

  let bugs: Bugs = res.json().await?;
  if let Some(bug) = bugs.bugs.first() {
    let mut e = EmbedBuilder::new()
      .title(&bug.summary)
      .url(format!("https://bugs.gentoo.org/{bug_number}"))
      .color(0xfd_69_b3)
      .footer(EmbedFooterBuilder::new(format!("Requested by {}", msg.author.name)));
    if !bug.creation_time.is_empty() {
      if let Ok(dt) = DateTime::parse_from_rfc3339(&bug.creation_time) {
        if let Ok(tt) = Timestamp::from_secs(dt.timestamp()) {
          e = e.timestamp(tt);
        }
      }
    }
    if !bug.assigned_to.is_empty() {
      e = e.field(EmbedFieldBuilder::new("assigned", &bug.assigned_to).inline());
    }
    if !bug.creator.is_empty() {
      e = e.field(EmbedFieldBuilder::new("creator", &bug.creator).inline());
    }
    if !bug.priority.is_empty() {
      e = e.field(EmbedFieldBuilder::new("priority", &bug.priority).inline());
    }
    if !bug.severity.is_empty() {
      e = e.field(EmbedFieldBuilder::new("severity", &bug.severity).inline());
    }
    if !bug.product.is_empty() {
      e = e.field(EmbedFieldBuilder::new("product", &bug.product).inline());
    }
    if !bug.resolution.is_empty() {
      e = e.field(EmbedFieldBuilder::new("resolution", &bug.resolution).inline());
    }
    if !bug.status.is_empty() {
      e = e.field(EmbedFieldBuilder::new("status", &bug.status).inline());
    }
    let embed = e.validate()?.build();
    state
      .http
      .create_message(msg.channel_id)
      .reply(msg.id)
      .embeds(&[embed])
      .await?;
  } else {
    state
      .http
      .create_message(msg.channel_id)
      .reply(msg.id)
      .content(&format!("Can't find bug {bug_number}"))
      .await?;
  }
  Ok(())
}
