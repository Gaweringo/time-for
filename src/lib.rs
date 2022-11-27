use std::{
    env::temp_dir,
    ffi::OsStr,
    fs,
    io::{self, Cursor},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::bail;
use arboard::Clipboard;
use chrono::Datelike;

use ordinal::Ordinal;
use serde::Deserialize;
use spinners::{Spinner, Spinners};
use tenor::TenorError;
use tfc::{Context, Key, KeyboardContext};

pub mod clapper;
pub mod ffmpeg;
mod secrets;
mod tenor;

struct MediaFile(PathBuf);

impl MediaFile {
    /// The given file but with "_text" added to the file name
    pub fn with_text(&self) -> PathBuf {
        self.add_to_file_name("_text")
    }

    /// The given file but with "_scaled" added to the file name
    pub fn scaled(&self) -> PathBuf {
        self.add_to_file_name("_scaled")
    }

    //
    pub fn base(&self) -> PathBuf {
        self.0.clone()
    }

    /// Adds the `addition` string to the file name of the `file`
    fn add_to_file_name(&self, addition: &str) -> PathBuf {
        self.0.with_file_name(
            (self.0.file_stem().unwrap_or_default().to_string_lossy()
                + addition
                + "."
                + self
                    .0
                    .extension()
                    .unwrap_or_else(|| OsStr::new("webm"))
                    .to_string_lossy())
            .to_string(),
        )
    }
}

use thiserror::Error;
#[derive(Error, Debug)]
pub enum TimeForError {
    #[error("ffmpeg could not be found in path")]
    FfmpegNotFound,
    #[error("could not run ffmpeg command")]
    FfmpegError {
        #[from]
        source: ffmpeg::FfmpegError,
    },
    #[error("could not create working directory")]
    CreateWorkingDirectory { source: std::io::Error },
    #[error("there was an error with a file")]
    Io {
        // #[from]
        source: std::io::Error,
    },
    #[error("there was an error downloading a gif")]
    Download {
        #[from]
        source: io::Error,
    },
    #[error("could not get a random gif")]
    GetRandGif {
        #[from]
        source: TenorError,
    },
    #[error("could not make web request")]
    Reqwest {
        #[from]
        source: reqwest::Error,
    },
    #[error("the gifs could not be scaled correctly!\nffmpeg exit code ({})", exit_code.map_or("None".to_string(), |c| c.to_string()))]
    ScalingError { exit_code: Option<i32> },
}

pub fn run(clap_args: clapper::Args) -> Result<(), TimeForError> {
    println!("TIME FOR");

    // let clap_args = clapper::Inputs::parse();

    if !ffmpeg::is_available() {
        return Err(TimeForError::FfmpegNotFound);
    }

    let query = &clap_args.query;

    let temp = temp_dir();
    let work_dir = temp.join("time-for");

    let query_file = MediaFile(work_dir.join("query.webm"));
    let look_at_time_file = MediaFile(work_dir.join("look_at_time.webm"));

    let final_output = work_dir.join("full.webm");

    // ?: Is the check even needed?
    fs::create_dir_all(work_dir).map_err(|e| TimeForError::CreateWorkingDirectory { source: e })?;

    ///// TODO: Maybe use https://crates.io/crates/indicatif instead
    // TODO: Look for a way to remove Spinner on error
    let mut sp = Spinner::with_timer(Spinners::Arc, "Creating GIF".into());

    //* Download a random gif
    if let Some(query) = query {
        let random_webm_url = tenor::random_webm(query, Some(clap_args.considered_gifs))?;
        download_file(&random_webm_url, &query_file.base())?;
    }

    let random_look_at_time_webm_url = tenor::random_webm("look at time", Some(16))?;
    download_file(&random_look_at_time_webm_url, &look_at_time_file.base())?;

    //* Scale to same size
    let mut handles = vec![];

    if query.is_some() {
        handles.push(ffmpeg::scale(
            &query_file.base(),
            None,
            &query_file.scaled(),
        ));
    }

    handles.push(ffmpeg::scale(
        &look_at_time_file.base(),
        None,
        &look_at_time_file.scaled(),
    ));

    for handle in handles {
        let output = handle.unwrap().wait_with_output()?;

        // TODO: Make this error handling better
        if !output.status.success() {
            return Err(TimeForError::ScalingError {
                exit_code: output.status.code(),
            });
        }
    }

    //* Create text for gif
    let time = chrono::Local::now() + chrono::Duration::seconds(clap_args.delay as i64);
    let day_ord = Ordinal(time.day()).to_string();
    let format_str = format!("It is %H:%M:%S %A %B {day_ord} %Y");
    let text = time.format(&format_str);

    let mut handles = vec![];

    //* Add text to time gif
    handles.push(ffmpeg::add_text(
        &look_at_time_file.scaled(),
        &text.to_string(),
        &look_at_time_file.with_text(),
    ));

    if let Some(query) = query {
        let query_text = clap_args
            .custom_text
            .unwrap_or(format!("time for {}", query));

        // Add text to query gif
        handles.push(ffmpeg::add_text(
            &query_file.scaled(),
            &query_text,
            &query_file.with_text(),
        ));
    }

    for handle in handles {
        handle
            .unwrap()
            .wait()
            .expect("Add text command wasn't running");
    }

    //* Stitch gifs
    if query.is_some() {
        ffmpeg::stitch_files_concat_demuxer(
            &look_at_time_file.with_text(),
            &query_file.with_text(),
            &final_output,
        )?;
    } else {
        fs::rename(&look_at_time_file.with_text(), &final_output)
            .expect("Rename look_at_time_text_scaled to final_output");
    }

    //* Upload file to imgur
    let res = upload_video_to_imgur(&final_output);

    sp.stop_with_newline();

    if output_and_paste(res).is_err() {
        eprintln!("There was an error uploading to imgur, so here is the file path instead:");
        // Print path to output file
        let can_path = final_output.as_os_str().to_string_lossy();
        eprintln!("{}", &can_path);
    }

    //* Open output folder in windows explorer if requested with "o" or "open"
    if clap_args.explorer {
        let explorer_arg = format!("/select,{}", final_output.as_os_str().to_string_lossy());
        Command::new("explorer").arg(explorer_arg).output().unwrap();
    }

    if clap_args.open {
        open::that(&final_output).expect("Open the file");
    }

    println!("DONE DONE DONE DONE DONE DONE");
    Ok(())
}

fn download_file(url: &str, file_path: &std::path::Path) -> Result<(), TimeForError> {
    let res = reqwest::blocking::get(url)?;
    let mut random_webm_file = fs::File::create(file_path)?;
    let mut content = Cursor::new(res.bytes()?);
    // let content = res.text()?;
    io::copy(&mut content, &mut random_webm_file)?;
    Ok(())
}

fn upload_video_to_imgur(file_path: &Path) -> reqwest::blocking::Response {
    let imgur_api = "https://api.imgur.com/3/upload";
    // let file = File::open(&file_path).expect("Open file");
    let form = reqwest::blocking::multipart::Form::new()
        .file("video", file_path)
        .expect("Create form with file");

    let client = reqwest::blocking::Client::new();
    let res = client
        .post(imgur_api)
        .header(
            "Authorization",
            format!("Client-ID {}", secrets::IMGUR_CLIENT_ID),
        )
        .multipart(form)
        .send()
        .unwrap();
    res
}

fn output_and_paste(res: reqwest::blocking::Response) -> anyhow::Result<()> {
    if let Ok(resp) = res.json::<Response>() {
        // Remove the dot '.' at the end of the link when uploading webm
        let link = resp.data.link.trim_end_matches('.');

        let mut clipboard = Clipboard::new().expect("Create new clipboard");
        let _ = clipboard.set_text(<&str>::clone(&link));

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
    data: Data,
}

#[derive(Deserialize, Debug)]
struct Data {
    link: String,
}
