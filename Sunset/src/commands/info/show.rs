use crate::{
  types::common::State,
  db::sled_info
};

use twilight_model::channel::Message;

pub async fn show(msg: Message, state: State) -> anyhow::Result<()> {
  tracing::debug!(
    "show command in channel {} by {}",
    msg.channel_id,
    msg.author.name
  );

  let mut key = String::new();
  for if_str in msg.content.split_whitespace() {
    if if_str.len() > 1 && if_str != "-show" {
      key = if_str.to_string();
      break;
    }
  }

  if key.is_empty() {
    state
      .http
      .create_message(msg.channel_id)
      .reply(msg.id)
      .content("You need to provide something to show")
      .await?;
    return Ok(())
  }

  match sled_info::read(&key).await {
    Ok(val) => {
      state
        .http
        .create_message(msg.channel_id)
        .reply(msg.id)
        .content(&val)
        .await?;
    }, Err(why) => {
      tracing::error!("Failed to get {key}, {why}");
    }
  }

  Ok(())
}
