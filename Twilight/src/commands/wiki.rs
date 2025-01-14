use crate::types::{
  common::State,
  gentoo::Wiki
};

use twilight_model::channel::Message;

use twilight_util::builder::embed::{
  EmbedBuilder,
  EmbedFieldBuilder,
  EmbedFooterBuilder
};

pub async fn wiki(msg: Message, state: State) -> anyhow::Result<()> {
  tracing::debug!(
    "wiki command in channel {} by {}",
    msg.channel_id,
    msg.author.name
  );

  let mut search_text = String::new();
  for if_str in msg.content.split_whitespace() {
    if if_str.len() > 1 && if_str != "-wiki" {
      search_text = if_str.to_string();
      break;
    }
  }

  if search_text.is_empty() {
    state
      .http
      .create_message(msg.channel_id)
      .reply(msg.id)
      .content("You need to provide something to search for")
      .await?;
    return Ok(())
  }

  let mut maybe_second = String::new();
  for if_str in msg.content.split_whitespace() {
    if if_str.len() > 1 && if_str != "-wiki" && if_str != search_text {
      maybe_second = if_str.to_string();
      break;
    }
  }

  let res = state.request_client.get(
    format!("https://wiki.gentoo.org/api.php?action=opensearch&search={search_text}")).send().await?;

  let (search_request, texts, _, links): Wiki = res.json().await?;

  if links.is_empty() {
    state
      .http
      .create_message(msg.channel_id)
      .reply(msg.id)
      .content(&format!("empty result for : {search_text}"))
      .await?;
    return Ok(());
  }

  let mut e = EmbedBuilder::new()
    .title(&search_request)
    .url( links.first().unwrap_or(&"https://wiki.gentoo.org".to_string()) )
    .color(0xfd_69_b3)
    .footer(EmbedFooterBuilder::new(format!("Requested by {}", msg.author.name)));

  let mut filtered_result = false;
  if !maybe_second.is_empty() {
    let other_lowered = maybe_second.to_lowercase();
    for (i, link) in links.iter().enumerate() {
      if let Some(title) = texts.get(i) {
        let title_lowered = title.to_lowercase();
        if title_lowered.contains(&other_lowered) {
          e = e.field(EmbedFieldBuilder::new(title, link));
          if !filtered_result { filtered_result = true; }
        }
      }
    }
  }

  if !filtered_result {
    for (i, link) in links.iter().enumerate() {
      if let Some(title) = texts.get(i) {
        e = e.field(EmbedFieldBuilder::new(title, link));
      }
    }
  }

  let embed = e.validate()?.build();
  state
    .http
    .create_message(msg.channel_id)
    .reply(msg.id)
    .embeds(&[embed])
    .await?;

  Ok(())
}
