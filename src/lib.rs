use std::{
    env::{args, temp_dir},
    error::Error,
    fs::{self, File},
    io::{self, Cursor},
    path::Path,
    process::{Command, Stdio},
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

pub fn run() -> Result<(), Box<dyn Error>> {
    println!("YES OW TIME");

    // TODO: use clap to get this from an argument
    let query_text = "going home";

    let temp = temp_dir();
    let output_file = temp.join("time-for\\time-for.gif");
    let query_gif_with_text = temp.join("time-for\\query_text.gif");
    let random_webm_file_path = temp.join("time-for\\query.webm");
    let look_at_time_webm_file_path = temp.join("time-for\\look_at_time_rnd.webm");
    let final_output_file = temp.join("time-for\\full.gif");

    if let Some(parent_dir) = output_file.parent() {
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
    download_file(&random_webm_url, &random_webm_file_path)?;

    let random_look_at_time_webm_url = tenor::random_webm("look at time", None)?;
    download_file(&random_look_at_time_webm_url, &look_at_time_webm_file_path)?;

    // Convert webm to gif
    let mut handle = convert_to_gif(&random_webm_file_path)?;
    let mut handle2 = convert_to_gif(&look_at_time_webm_file_path)?;

    handle.wait()?;
    handle2.wait()?;

    // Create text for gif
    let day_ord = Ordinal(chrono::Local::now().day()).to_string();
    let format_str = format!("It is %H\\:%M\\:%S %A %B {day_ord} %Y");
    let text = chrono::Local::now().format(&format_str);

    // Add text to time gif
    add_text(
        &look_at_time_webm_file_path,
        &text.to_string(),
        &output_file,
    );

    // Add text to query gif
    add_text(
        &random_webm_file_path,
        format!("time for {}", query_text).as_str(),
        &query_gif_with_text,
    );

    // Stitch gifs
    Command::new("ffmpeg")
        .arg("-i")
        .arg(&output_file)
        .arg("-i")
        .arg(query_gif_with_text)
        .args([
            "-filter_complex",
            "[0:v] [1:v] concat=n=2:v=1:unsafe=true [v]",
        ])
        .args(["-map", "[v]"])
        .arg(&final_output_file)
        .arg("-y")
        // .stderr(Stdio::inherit())
        .output()?;

    // Upload file to imgur
    let res = upload_to_imgur(&final_output_file);

    // tx.send(true).expect("Send stop signal to loading thread");
    sp.stop_with_newline();
    // loading_thread_handle.join().expect("To join thread");

    if let Err(_) = output_and_paste(res) {
        eprintln!("There was an error uploading to imgur, so here is the file path instead:");
        // Print path to output file
        let can_path = final_output_file.as_os_str().to_string_lossy();
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
                    final_output_file.as_os_str().to_string_lossy()
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

fn add_text(input_file: &std::path::PathBuf, text: &str, output_file: &std::path::PathBuf) {
    let vf_text = format!("drawtext=fontfile=assets/Montserrat-Bold.ttf:fontcolor=white:borderw=3:fontsize=22:x=(w-text_w)/2:y=(h-text_h)-20:text='{}'", text);
    Command::new("ffmpeg")
        .arg("-i")
        .arg(&input_file.with_extension("gif"))
        .args(["-vf", &vf_text])
        .arg("-y")
        .arg(&output_file.as_os_str())
        // .stdout(Stdio::inherit())
        // .stderr(Stdio::inherit())
        .output()
        .expect("error ffmpeg run");
}

/// Converts the given file to a gif and and returns the handle to the spawned child process.
///
/// # Errors
///
/// This function will return an error if there is an error spawning the child.
fn convert_to_gif(
    random_webm_file_path: &std::path::PathBuf,
) -> Result<std::process::Child, Box<dyn Error>> {
    let handle = Command::new("ffmpeg")
        .arg("-i")
        .arg(&random_webm_file_path.as_os_str())
        .arg("-y")
        .args([
            "-filter_complex",
            "[0:v] split [a][b];[a] palettegen [p];[b][p] paletteuse",
        ])
        .arg(&random_webm_file_path.with_extension("gif"))
        .stderr(Stdio::null())
        .spawn()?;
    Ok(handle)
}

fn download_file(url: &str, file_path: &std::path::PathBuf) -> Result<(), Box<dyn Error>> {
    let res = reqwest::blocking::get(url)?;
    let mut random_webm_file = fs::File::create(file_path.clone())?;
    let mut content = Cursor::new(res.bytes()?);
    io::copy(&mut content, &mut random_webm_file)?;
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

fn output_and_paste(res: reqwest::blocking::Response) -> anyhow::Result<()> {
    if let Ok(resp) = res.json::<Response>() {
        let mut clipboard = Clipboard::new().expect("Create new clipboard");
        let _ = clipboard.set_text(resp.data.link.clone());

        let mut ctx = Context::new()?;
        ctx.key_down(Key::Control)?;
        ctx.key_click(Key::V)?;
        ctx.key_up(Key::Control)?;
        println!("{:#?}", resp.data.link);
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
