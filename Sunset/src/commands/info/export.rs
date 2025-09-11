use crate::{
  types::common::State,
  db::sled_info
};

use std::fs;

use twilight_model::channel::Message;

pub async fn export(msg: Message, state: State) -> anyhow::Result<()> {
  tracing::debug!(
    "export command in channel {} by {}",
    msg.channel_id,
    msg.author.name
  );

  match sled_info::export().await {
    Ok(data) => {
      let json = serde_json::to_string_pretty(&data).unwrap();
      match fs::write("sled_data.json", json) {
        Ok(()) => {
          state
          .http
          .create_message(msg.channel_id)
          .reply(msg.id)
          .content("database exported")
          .await?;
        }, Err(why) => {
          tracing::error!("Failed to export sled db, {why}");
        }
      }
    }, Err(why) => {
      tracing::error!("Failed to export sled db, {why}");
    }
  }
  Ok(())
}

pub async fn import(msg: Message, state: State) -> anyhow::Result<()> {
  tracing::debug!(
    "import command in channel {} by {}",
    msg.channel_id,
    msg.author.name
  );

  match fs::read_to_string("sled_data.json") {
    Ok(json) => {
      let data_imported: Vec<(String, String)> =
        serde_json::from_str(&json).unwrap_or(vec![]);
  
      match sled_info::import(data_imported).await {
        Ok(()) => {
          state
            .http
            .create_message(msg.channel_id)
            .reply(msg.id)
            .content("database imported")
            .await?;
        }, Err(why) => {
          tracing::error!("Failed to show info list, {why}");
        }
      }
    }, Err(why) => {
      tracing::error!("Failed to import sled db, {why}");
    }
  }

  Ok(())
}
