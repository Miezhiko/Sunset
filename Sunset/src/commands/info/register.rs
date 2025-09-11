use crate::{
  types::common::State,
  db::sled_info
};

use twilight_model::channel::Message;

pub async fn register(msg: Message, state: State) -> anyhow::Result<()> {
  tracing::debug!(
    "register command in channel {} by {}",
    msg.channel_id,
    msg.author.name
  );

  let mut key = String::new();
  for if_str in msg.content.split_whitespace() {
    if if_str.len() > 1 && if_str != "-register" {
      key = if_str.to_string();
      break;
    }
  }

  if key.is_empty() {
    state
      .http
      .create_message(msg.channel_id)
      .reply(msg.id)
      .content("You need to provide something to register")
      .await?;
    return Ok(())
  }

  let skip_len = "-register".len() + key.len() + 2;
  let rest = &msg.content[skip_len..msg.content.len()];

  if rest.is_empty() {
    state
      .http
      .create_message(msg.channel_id)
      .reply(msg.id)
      .content("You need to provide text for a key after it")
      .await?;
    return Ok(())
  }

  if let Err(why) = sled_info::store(&key, rest).await {
    tracing::error!("Failed to register {key}, {why}");
  } else {
    state
      .http
      .create_message(msg.channel_id)
      .reply(msg.id)
      .content(&format!("Registered {key}"))
      .await?;
  }

  Ok(())
}
