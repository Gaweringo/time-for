# time-for
A CLI program to generate GIFs containing the current time and the specified topic.

# Installation
## Prerequisites
- ffmpeg
## Setup
Create `src/secrets.rs` like so:
```rust
pub static TENOR_API_KEY: &str = "API_KEY";
pub static IMGUR_CLIENT_ID: &str = "CLIENT_ID";
```
- Tenor API: https://developers.google.com/tenor/guides/quickstart
- Imgur ClientId: https://imgur.com/account/settings/apps

## Installation
Then install with `cargo install --path .`
