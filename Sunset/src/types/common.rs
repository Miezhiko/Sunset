use std::sync::Arc;
use std::collections::HashSet;

use serde::Deserialize;

use twilight_http::Client as HttpClient;
use twilight_model::id::{ Id, marker::GuildMarker };

use reqwest::Client as Reqwest;

#[derive(Clone, Deserialize, Debug)]
pub struct IOptions {
  pub discord: String,
  pub allowed_guilds: Vec<u64>
}

#[derive(Debug)]
pub struct StateRef {
  pub http: HttpClient,
  pub request_client: Reqwest,
  pub allowed_guilds: HashSet<Id<GuildMarker>>
}

pub type State = Arc<StateRef>;
