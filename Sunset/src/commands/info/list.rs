use crate::{
  types::common::State,
  db::sled_info
};

use twilight_model::channel::Message;

pub async fn list(msg: Message, state: State) -> anyhow::Result<()> {
  tracing::debug!(
    "list command in channel {} by {}",
    msg.channel_id,
    msg.author.name
  );

  match sled_info::list().await {
    Ok(val) => {
      state
        .http
        .create_message(msg.channel_id)
        .reply(msg.id)
        .content(&val)
        .await?;
    }, Err(why) => {
      tracing::error!("Failed to show info list, {why}");
    }
  }

  Ok(())
}
