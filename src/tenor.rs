use rand::{thread_rng, Rng};
use serde::Deserialize;
use thiserror::Error;

use crate::secrets::TENOR_API_KEY;

/// Structure for the parameters used in a Tenor search request
#[derive(serde::Serialize)]
struct SearchRequest {
    /// The query string a.k.a search term
    q: String,
    /// The api key for Tenor
    key: String,
    /// The maximum number of gifs to be returned
    limit: u8,
    /// The offset to get the `limit` number of gifs from
    offset: Option<usize>,
}

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
/// This function will return an error if the request to the TenorApi,
/// the json parsing of the response was unsuccessful or there was no gif found
pub fn random_webm(
    search_term: &str,
    considered_gifs: Option<u8>,
    // min_duration: Option<u32>,
) -> Result<String, TenorError> {
    let tenor_url = "https://tenor.googleapis.com/v2/search";
    let considered_gifs = considered_gifs.unwrap_or(10);
    let search_request = SearchRequest {
        q: search_term.to_string(),
        key: TENOR_API_KEY.to_string(),
        limit: considered_gifs.into(),
        offset: None,
    };

    let client = reqwest::blocking::Client::new();
    let res = client
        .request(reqwest::Method::GET, tenor_url)
        .query(&search_request)
        .send()?;

    // Done: Check for error response like https://stackoverflow.com/a/61429476/10018101
    // Return correct error if error or normal (just like currently) otherwise
    let body: ResponseType = res.json()?;

    match body {
        ResponseType::Err(e) => {
            return Err(TenorError::TenorApi {
                code: e.error.code,
                message: e.error.message,
            });
        }
        ResponseType::Ok(body) => {
            let max_gif = if (body.results.len() as u8) < considered_gifs {
                body.results.len() as u8
            } else {
                considered_gifs
            };

            if max_gif == 0 {
                return Err(TenorError::NoGifFound {
                    query: search_request.q,
                });
            }

            let mut rng = thread_rng();
            let selected_gif = rng.gen_range(0..max_gif) as usize;

            return Ok(body.results[selected_gif as usize]
                .media_formats
                .webm
                .url
                .clone());
        }
    }
}

#[derive(Error, Debug)]
pub enum TenorError {
    #[error("Could not find a GIF for query: \"{query}\"")]
    NoGifFound { query: String },
    #[error("There was an error with the request:\n{source:?}")]
    Request { source: reqwest::Error },
    #[error("There was an error with the response:\n{source:?}")]
    Response { source: reqwest::Error },
    #[error("Tenor responded with the error {code}: {message:?}")]
    TenorApi { code: u32, message: String },
}

impl From<reqwest::Error> for TenorError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_body() || err.is_decode() || err.is_timeout() || err.is_status() {
            TenorError::Response { source: err }
        } else {
            TenorError::Request { source: err }
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ResponseType {
    Ok(Body),
    Err(ErrorBody),
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
    webm: Webm,
}

#[derive(Deserialize, Debug)]
struct Webm {
    url: String,
    // Duration is only non zero for mp4
    // duration: u32,
}

#[derive(Deserialize, Debug)]
struct ErrorBody {
    error: ResponseError,
}

#[derive(Deserialize, Debug)]
struct ResponseError {
    code: u32,
    message: String,
}
