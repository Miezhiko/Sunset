use crate::{
  types::common::State,
  commands::{
    bug::bug,
    wiki::wiki,
    overlays::overlays,
    info::{
      register::register,
      show::show,
      list::list,
      delete::delete,
      export
    }
  }
};

use std::{
  error::Error,
  sync::Arc,
  future::Future
};

use once_cell::sync::Lazy;
use regex::Regex;

use twilight_gateway::Event;

use twilight_model::{
  channel::Message,
  gateway::payload::incoming::MessageCreate
};

const ME: u64 = 510368731378089984;

fn spawn(fut: impl Future<Output = anyhow::Result<()>> + Send + 'static) {
  tokio::spawn(async move {
    if let Err(why) = fut.await {
      tracing::debug!("handler error: {why:?}");
    }
  });
}

async fn help(msg: Message, state: State) -> anyhow::Result<()> {
  tracing::debug!(
    "help command in channel {} by {}",
    msg.channel_id,
    msg.author.name
  );
  state
    .http
    .create_message(msg.channel_id)
    .reply(msg.id)
    .content("I can handle -bug <num> command, and maybe -overlays <search>, and -wiki <search>")
    .await?;
  Ok(())
}

fn contains_bug(text: &str) -> Option<i32> {
  static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)bug (\d+)").unwrap());
  let cap = RE.captures(text)?;
  if cap.len() > 1 {
    let number = cap[1].parse::<i32>().ok()?;
    Some(number)
  } else {
    None
  }
}

async fn handle_message(
  msg: Box<MessageCreate>,
  state: &State
) -> Result<(), Box<dyn Error + Send + Sync>> {
  if msg.author.bot {
    return Ok(());
  }

  match msg.content.split_whitespace().next() {
    Some("-help")     => spawn(help(msg.0, Arc::clone(state))),
    Some("-bug")      => spawn(bug(msg.0, None, Arc::clone(state))),
    Some("-wiki")     => spawn(wiki(msg.0, Arc::clone(state))),
    Some("-overlays") => spawn(overlays(msg.0, Arc::clone(state))),
    Some("-register") => spawn(register(msg.0, Arc::clone(state))),
    Some("-show")     => spawn(show(msg.0, Arc::clone(state))),
    Some("-list")     => spawn(list(msg.0, Arc::clone(state))),
    Some("-delete")   => spawn(delete(msg.0, Arc::clone(state))),
    Some(cmd)         => {
      if msg.author.id.get() == ME {
        match cmd {
          "-import"  => spawn(export::import(msg.0, Arc::clone(state))),
          "-export"  => spawn(export::export(msg.0, Arc::clone(state))),
          _category  => {}
        }
      } else if let Some(bug_number) = contains_bug(msg.content.as_str()) {
        spawn(bug(msg.0, Some(bug_number), Arc::clone(state)))
      }
    },
    None              => {}
  };

  Ok(())
}

pub async fn handle_event(
  event: Event,
  state: State,
) -> Result<(), Box<dyn Error + Send + Sync>> {
  match event {
    Event::GuildCreate(guild) => {
      if !state.allowed_guilds.contains(&guild.id()) {
        state.http.leave_guild(guild.id()).await?;
        tracing::info!("Left unallowed guild: {}", guild.id());
      }
      Ok(())
    },
    Event::MessageCreate(msg) => handle_message(msg, &state).await,
    Event::Ready(_) => {
      tracing::info!("Shard is ready");
      Ok(())
    }
    _ => Ok(())
  }
}
