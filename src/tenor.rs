use std::error::Error;

use rand::{Rng, thread_rng};
use serde::Deserialize;

use crate::secrets::TENOR_API_KEY;

/// Randomly picks one of the first `considered_gifs` for the given `search_term`.
/// The default value for `considered_gifs` is set to 10.
///
/// # Examples
/// 
/// ```rust
/// let random_gif_url = random_gif("Overwatch time", Some(5)).unwrap();
/// ```
/// # Errors
///
/// This function will return an error if the request to the TenorApi
/// or the json parsing of the response was unsuccessful
pub fn random_webm(
    search_term: &str,
    considered_gifs: Option<u8>,
) -> Result<String, Box<dyn Error>> {
    let tenor_url = format!("https://tenor.googleapis.com/v2/search");
    let considered_gifs = considered_gifs.unwrap_or(10);
    let client = reqwest::blocking::Client::new();
    let res = client
        .request(reqwest::Method::GET, tenor_url)
        .query(&[
            ("q", search_term),
            ("key", TENOR_API_KEY),
            ("limit", &considered_gifs.to_string()),
        ])
        .send()?;

    let mut rng = thread_rng();
    let selected_gif = rng.gen_range(0..considered_gifs);
    let body: Body = res.json()?;
    // println!("{:#?}", body);
    return Ok(body.results[selected_gif as usize].media_formats.webm.url.clone());
}

#[derive(Deserialize, Debug)]
struct Body {
    results: Vec<MediaFormats>,
}

#[derive(Deserialize, Debug)]
struct MediaFormats {
    media_formats: WebmUrl,
}

#[derive(Deserialize, Debug)]
struct WebmUrl {
    webm: Url,
}

#[derive(Deserialize, Debug)]
struct Url {
    url: String,
}
