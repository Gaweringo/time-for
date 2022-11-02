use std::{
    env::{args, temp_dir},
    error::Error,
    fs::{self},
    io::{self, Cursor},
    path::Path,
    process::{Command},
};

use anyhow::bail;
use arboard::Clipboard;
use chrono::Datelike;
use ordinal::Ordinal;
use serde::Deserialize;
use spinners::{Spinner, Spinners};
use tfc::{Context, Key, KeyboardContext};

mod secrets;
mod tenor;
mod ffmpeg;

pub fn run() -> Result<(), Box<dyn Error>> {
    println!("TIME FOR");

    // TODO: use clap to get this from an argument
    let query_text = "sleep";

    let temp = temp_dir();
    let query_webm = temp.join("time-for\\query.webm");
    let query_webm_with_text = temp.join("time-for\\query_text.webm");
    let query_webm_with_text_scaled = temp.join("time-for\\query_text_scaled.webm");
    
    let look_at_time_webm = temp.join("time-for\\look_at_time_rnd.webm");
    let look_at_time_text_webm = temp.join("time-for\\time-for.webm");
    let look_at_time_text_scaled_webm = temp.join("time-for\\time-for_scaled.webm");
    let final_output = temp.join("time-for\\full.webm");

    if let Some(parent_dir) = look_at_time_text_webm.parent() {
        match parent_dir.try_exists() {
            Ok(false) | Err(_) => fs::create_dir_all(parent_dir)?,
            Ok(true) => {}
        }
    }

    let mut sp = Spinner::new(Spinners::Arc, "Creating GIF".into());
    // let (tx, rx) = mpsc::channel();
    // let loading_thread_handle = thread::spawn(move || {
    //     print!("Creating GIF");
    //     loop {
    //         match rx.try_recv() {
    //             Ok(stop) if stop => break,
    //             Err(_) | Ok(_) => {
    //                 print!(".");
    //                 io::stdout().flush().unwrap();
    //                 thread::sleep(Duration::from_millis(200));
    //             }
    //         }
    //     }
    //     println!();
    // });

    // Download a random gif
    let random_webm_url = tenor::random_webm(query_text, Some(5))?;
    // println!("{}", random_webm_url);
    download_file(&random_webm_url, &query_webm)?;

    let random_look_at_time_webm_url = tenor::random_webm("look at time", None)?;
    download_file(&random_look_at_time_webm_url, &look_at_time_webm)?;

    // Convert webm to gif
    // let mut handle = ffmpeg::convert_to_gif(&random_webm_file_path)?;
    // let mut handle2 = ffmpeg::convert_to_gif(&look_at_time_webm_file_path)?;

    // handle.wait()?;
    // handle2.wait()?;


    
    // Create text for gif
    let day_ord = Ordinal(chrono::Local::now().day()).to_string();
    let format_str = format!("It is %H\\:%M\\:%S %A %B {day_ord} %Y");
    let text = chrono::Local::now().format(&format_str);
    
    let mut handles = vec![];

    // Add text to time gif
    handles.push(ffmpeg::add_text(
        &look_at_time_webm,
        &text.to_string(),
        &look_at_time_text_webm,
    ));

    // Add text to query gif
    handles.push(ffmpeg::add_text(
        &query_webm,
        format!("time for {}", query_text).as_str(),
        &query_webm_with_text,
    ));

    for handle in handles {
        handle.unwrap().wait().expect("Add text command wasn't running");
    }

    // Scale to same size
    let mut handles = vec![];

    handles.push(ffmpeg::scale(&query_webm_with_text, None, &query_webm_with_text_scaled));
    handles.push(ffmpeg::scale(&look_at_time_text_webm, None, &look_at_time_text_scaled_webm));

    for handle in handles {
        handle.unwrap().wait().expect("Add text command wasn't running");
    }

    // Stitch gifs
    ffmpeg::stitch_files_concat_demuxer(&look_at_time_text_scaled_webm, &query_webm_with_text_scaled, &final_output)?;

    // Upload file to imgur
    let res = upload_video_to_imgur(&final_output);

    // tx.send(true).expect("Send stop signal to loading thread");
    sp.stop_with_newline();
    // loading_thread_handle.join().expect("To join thread");

    if let Err(_) = output_and_paste(res) {
        eprintln!("There was an error uploading to imgur, so here is the file path instead:");
        // Print path to output file
        let can_path = final_output.as_os_str().to_string_lossy();
        eprintln!("{}", &can_path);
    }

    // Open output folder in windows explorer if requested with "o" or "open"
    // TODO: Use clap to make this parsing better
    let mut args = args();
    args.next();
    match args.next() {
        Some(arg) => match arg.as_str() {
            "o" | "open" => {
                let explorer_arg = format!(
                    "/select,{}",
                    final_output.as_os_str().to_string_lossy()
                );
                Command::new("explorer").arg(explorer_arg).output().unwrap();
            }
            _ => {}
        },
        _ => {}
    }

    println!("DONE DONE DONE DONE DONE DONE");
    Ok(())
}

fn download_file(url: &str, file_path: &std::path::PathBuf) -> Result<(), Box<dyn Error>> {
    let res = reqwest::blocking::get(url)?;
    let mut random_webm_file = fs::File::create(file_path.clone())?;
    let mut content = Cursor::new(res.bytes()?);
    io::copy(&mut content, &mut random_webm_file)?;
    Ok(())
}

fn upload_video_to_imgur(file_path: &Path) -> reqwest::blocking::Response {
    let imgur_api = "https://api.imgur.com/3/upload";
    // let file = File::open(&file_path).expect("Open file");
    let form = reqwest::blocking::multipart::Form::new().file("video", &file_path).expect("Create form with file");

    let client = reqwest::blocking::Client::new();
    let res = client
        .post(imgur_api)
        .header("Authorization", format!("Client-ID {}", secrets::CLIENT_ID))
        .multipart(form)
        .send()
        .unwrap();
    res
}

fn output_and_paste(res: reqwest::blocking::Response) -> anyhow::Result<()> {
    if let Ok(resp) = res.json::<Response>() {
        // Remove the dot '.' at the end of the link when uploading webm
        let link = &resp.data.link.trim_end_matches('.');

        let mut clipboard = Clipboard::new().expect("Create new clipboard");
        let _ = clipboard.set_text(link.clone());

        let mut ctx = Context::new()?;
        ctx.key_down(Key::Control)?;
        ctx.key_click(Key::V)?;
        ctx.key_up(Key::Control)?;
        println!("{}", link);
        return Ok(());
    } else {
        bail!("Output and paste error");
    }
}

#[derive(Deserialize)]
struct Response {
    data: GifUrl,
}
#[derive(Deserialize, Debug)]
struct GifUrl {
    link: String,
}
