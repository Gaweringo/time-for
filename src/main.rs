use std::{
    env::args,
    fs::{self, File},
    process::{Command},
};

use arboard::Clipboard;
use chrono::Datelike;
use ordinal::Ordinal;
use serde::Deserialize;
mod secrets;

fn main() {
    println!("YES OW TIME");

    let output_file = "assets/ow-time.gif";
    // Create text for gif
    let day_ord = Ordinal(chrono::Local::now().day()).to_string();
    let format_str = format!("It is %H\\:%M\\:%S %A %B {day_ord} %Y");
    let text = chrono::Local::now().format(&format_str);
    let vf_text = format!("drawtext=fontfile=assets/Montserrat-Bold.ttf:fontcolor=white:borderw=3:fontsize=22:x=(w-text_w)/2:y=(h-text_h)-20:text='{}'", text);

    println!("Creating GIF...");
    Command::new("ffmpeg")
        .args(["-i", "assets/look_at_time.gif"])
        .args(["-vf", vf_text.as_str()])
        .arg("-y")
        .arg(&output_file)
        // .stdout(Stdio::inherit())
        // .stderr(Stdio::inherit())
        .output()
        .expect("error ffmpeg run");

    // Open output folder in windows explorer if requested with "o" or "open"
    args().next();
    match args().next() {
        Some(arg) => match arg.as_str() {
            "o" | "open" => {
                Command::new("explorer").arg("assets").output().unwrap();
            }
            _ => {}
        },
        _ => {}
    }

    // Upload file to imgur
    let imgur_api = "https://api.imgur.com/3/image";
    let file = File::open(output_file).expect("Open file");

    let client = reqwest::blocking::Client::new();
    let res = client
        .post(imgur_api)
        .header("Authorization", format!("Client-ID {}", secrets::CLIENT_ID))
        .body(file)
        .send()
        .unwrap();

    // Ad link to clipboard or link to file if upload unsuccessful
    if let Ok(resp) = res.json::<Response>() {
        let mut clipboard = Clipboard::new().expect("Create new clipboard");
        let _ = clipboard.set_text(resp.data.link.clone());
        println!("{:#?}", resp.data.link);
    } else {
        eprintln!("There was an error uploading to imgur, so here is the file path instead:");
        // Print path to output file
        let can_path = fs::canonicalize("assets/ow-time.gif")
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap();
        eprintln!("{}", &can_path[4..]);
    }

    println!("DONE DONE DONE DONE DONE DONE");
}

#[derive(Deserialize)]
struct Response {
    data: GifUrl,
}
#[derive(Deserialize, Debug)]
struct GifUrl {
    link: String,
}
