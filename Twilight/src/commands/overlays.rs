use crate::types::common::State;

use twilight_model::channel::Message;

use tokio::task;

use nipper::Document;

use twilight_util::builder::embed::{
  EmbedBuilder,
  EmbedFooterBuilder
};

pub async fn overlays(msg: Message, state: State) -> anyhow::Result<()> {
  tracing::debug!(
    "overlays command in channel {} by {}",
    msg.channel_id,
    msg.author.name
  );

  let mut search_string = String::new();
  for if_str in msg.content.split_whitespace() {
    if if_str.len() > 1 && if_str != "-overlays" {
      search_string = if_str.to_string();
      break;
    }
  }

  if search_string.is_empty() {
    state
      .http
      .create_message(msg.channel_id)
      .reply(msg.id)
      .content("You need to provide something to search for")
      .await?;
    return Ok(())
  }

  let url = format!("http://gpo.zugaina.org/Search?search={}", &search_string);
  let resp = state.request_client.get(&url)
                  .send()
                  .await?
                  .text()
                  .await?;

  let resp_clone = resp.clone();
  let mut pages = task::spawn_blocking(move || -> usize {
    let document = Document::from(&resp_clone);
    let pages = document.nip("div[id=\"contentInner\"] > div[class=\"pager\"] > a[href]");
    // div 2 because there are pages on top and bottom and they look same
    pages.size() / 2
  }).await?;

  let mut top_level = vec![];
  let mut result_vec = vec![];
  let search_local = search_string.clone();
  result_vec.push(
    task::spawn_blocking(move || -> Vec<(String, String, String)> {
      let document = Document::from(&resp);
      document.nip("a > div").iter().take(5).flat_map(|element| {
        let text = element.text();
        let (atom, description)   = text.split_once(' ')?;
        let (_category, pkgname)  = atom.split_once('/')?;
        if pkgname.contains(&search_local) {
          Some((
            atom.to_string(),
            format!("http://gpo.zugaina.org/{atom}"),
            format!("**[{atom}](http://gpo.zugaina.org/{atom})** {description}")
          ))
        } else { None }
      }).collect::<Vec<(String, String, String)>>()
    }).await?
  );

  if pages > 0 {
    // it's hard to get all the pages from start so let take like first 30 pages
    // we will stop processing once we will get no results on the page
    if pages > 7 {
      pages = 30;
    }
    for p in 0..pages {
      let page = p + 2;
      let urlx = format!("https://gpo.zugaina.org/Search?search={}&use=&page={page}", &search_string);
      let respx = state.request_client.get(&urlx)
                                      .send()
                                      .await?
                                      .text()
                                      .await?;
      let search_local = search_string.clone();
      let page_results =
        task::spawn_blocking(move || -> Vec<(String, String, String)> {
          let document = Document::from(&respx);
          document.nip("a > div").iter().take(5).flat_map(|element| {
            let text = element.text();
            let (atom, description)   = text.split_once(' ')?;
            let (_category, pkgname)  = atom.split_once('/')?;
            if pkgname.contains(&search_local) {
              Some((
                atom.to_string(),
                format!("http://gpo.zugaina.org/{atom}"),
                format!("**[{atom}](http://gpo.zugaina.org/{atom})** {description}")
              ))
            } else { None }
          }).collect::<Vec<(String, String, String)>>()
        }).await?;
      if page_results.is_empty() {
        break;
      }
      result_vec.push(page_results);
    }
  };

  for tlv in &mut result_vec {
    top_level.append(tlv);
  }

  let mut parse_result = vec![];
  for (atom, pkg_url, desc) in top_level {
    let pkg_resp = state.request_client.get(&pkg_url).send().await?.text().await?;
    let pkg_level = task::spawn_blocking(move || -> Vec<String> {
      let document = Document::from(&pkg_resp);
      document.nip("div > li").iter().take(5).flat_map(|element| {
        let text  = element.text();
        #[allow(clippy::manual_pattern_char_comparison)]
        let split = text.split(|c| c == ' ' || c == '\n' || c == '\t')
                        .filter(|&x| !x.is_empty())
                        .collect::<Vec<&str>>();
        if split.is_empty() {
          None
        } else {
          let first = split.first()?;
          let last  = split.last()?;
          Some(format!(" • **{first}** from [{last}](https://data.gpo.zugaina.org/{last}/{atom})"))
        }
      }).collect::<Vec<String>>()
    }).await?;
    let pkg_level_str = pkg_level.join("\n");
    parse_result.push( format!("{desc}\n{pkg_level_str}") );
  }

  // TODO: if exact package name found then show only that
  if parse_result.is_empty() {
    state
      .http
      .create_message(msg.channel_id)
      .reply(msg.id)
      .content(&format!("empty result for : {search_string}"))
      .await?;
    return Ok(());
  }

  let parse_result_str = parse_result.join("\n\n");

  let parsed_many = parse_result_str.as_bytes()
    .chunks(4000)
    .map(|s| unsafe { ::std::str::from_utf8_unchecked(s) })
    .collect::<Vec<&str>>();

  for parsed_one in parsed_many {
    let embed = EmbedBuilder::new()
      .title(&search_string)
      .url(&url)
      .color(0xfd_69_b3)
      .description(parsed_one)
      .footer(EmbedFooterBuilder::new(format!("Requested by {}", msg.author.name)))
      .validate()?.build();
    state
      .http
      .create_message(msg.channel_id)
      .reply(msg.id)
      .embeds(&[embed])
      .await?;
  }

  Ok(())
}
