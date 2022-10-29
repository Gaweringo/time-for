use std::{
    env::{args, temp_dir},
    error::Error,
    fs::{self, File},
    path::Path,
    process::Command,
    sync::mpsc,
    thread,
    time::Duration, io::{self, Write},
};

use arboard::Clipboard;
use chrono::Datelike;
use ordinal::Ordinal;
use serde::Deserialize;
use spinners::{Spinner, Spinners};
use tfc::{Context, Key, KeyboardContext};

mod secrets;

pub fn run() -> Result<(), Box<dyn Error>> {
    println!("YES OW TIME");

    let temp = temp_dir();
    let output_file = temp.join("time-for.gif");

    // Create text for gif
    let day_ord = Ordinal(chrono::Local::now().day()).to_string();
    let format_str = format!("It is %H\\:%M\\:%S %A %B {day_ord} %Y");
    let text = chrono::Local::now().format(&format_str);
    let vf_text = format!("drawtext=fontfile=assets/Montserrat-Bold.ttf:fontcolor=white:borderw=3:fontsize=22:x=(w-text_w)/2:y=(h-text_h)-20:text='{}'", text);

    // println!("Creating GIF...");
    // let mut sp = Spinner::new(Spinners::SimpleDots, "Creating GIF".into());
    let (tx, rx) = mpsc::channel();
    let loading_thread_handle = thread::spawn(move || {
        print!("Creating GIF");
        loop {
            match rx.try_recv() {
                Ok(stop) if stop => break,
                Err(_) | Ok(_) => {
                    print!(".");
                    io::stdout().flush().unwrap();
                    thread::sleep(Duration::from_millis(200));
                }
            }
        }
        println!();
    });

    Command::new("ffmpeg")
        .args(["-i", "assets/look_at_time.gif"])
        .args(["-vf", vf_text.as_str()])
        .arg("-y")
        .arg(&output_file.as_os_str())
        // .stdout(Stdio::inherit())
        // .stderr(Stdio::inherit())
        .output()
        .expect("error ffmpeg run");

    // Upload file to imgur
    let res = upload_to_imgur(&output_file);

    tx.send(true).expect("Send stop signal to loading thread");
    // sp.stop_with_newline();
    loading_thread_handle.join().expect("To join thread");


    output_and_paste(res)?;

    // Open output folder in windows explorer if requested with "o" or "open"
    let mut args = args();
    args.next();
    match args.next() {
        Some(arg) => match arg.as_str() {
            "o" | "open" => {
                Command::new("explorer").arg("assets").output().unwrap();
            }
            _ => {}
        },
        _ => {}
    }

    println!("DONE DONE DONE DONE DONE DONE");
    Ok(())
}

fn upload_to_imgur(output_file: &Path) -> reqwest::blocking::Response {
    let imgur_api = "https://api.imgur.com/3/image";
    let file = File::open(output_file).expect("Open file");
    let client = reqwest::blocking::Client::new();
    let res = client
        .post(imgur_api)
        .header("Authorization", format!("Client-ID {}", secrets::CLIENT_ID))
        .body(file)
        .send()
        .unwrap();
    res
}

fn output_and_paste(res: reqwest::blocking::Response) -> Result<(), Box<dyn Error>> {
    Ok(if let Ok(resp) = res.json::<Response>() {
        let mut clipboard = Clipboard::new().expect("Create new clipboard");
        let _ = clipboard.set_text(resp.data.link.clone());

        let mut ctx = Context::new()?;
        ctx.key_down(Key::Control)?;
        ctx.key_click(Key::V)?;
        ctx.key_up(Key::Control)?;
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
    })
}

#[derive(Deserialize)]
struct Response {
    data: GifUrl,
}
#[derive(Deserialize, Debug)]
struct GifUrl {
    link: String,
}
