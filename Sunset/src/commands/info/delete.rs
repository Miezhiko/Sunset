use crate::{
  types::common::State,
  db::sled_info
};

use twilight_model::channel::Message;

pub async fn delete(msg: Message, state: State) -> anyhow::Result<()> {
  tracing::debug!(
    "delete command in channel {} by {}",
    msg.channel_id,
    msg.author.name
  );

  let mut key = String::new();
  for if_str in msg.content.split_whitespace() {
    if if_str.len() > 1 && if_str != "-delete" {
      key = if_str.to_string();
      break;
    }
  }

  match sled_info::delete(&key).await {
    Ok(_) => {
      state
        .http
        .create_message(msg.channel_id)
        .reply(msg.id)
        .content(&format!("Deleted {key}"))
        .await?;
    }, Err(why) => {
      tracing::error!("Failed to delete {key}, {why}");
    }
  }

  Ok(())
}
