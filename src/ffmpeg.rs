use std::{
    env::temp_dir,
    fs::{self, File},
    io::{self, Write},
    path::Path,
    process::{Child, Command, Stdio},
};
use thiserror::Error;

/// Overlays the `text` on the bottom of the `input_file` and saves it to the `output_file`.
/// It also scales the file to 480x270 so that all files have the same size and can be
/// stitched together faster.
///
/// # Panics
///
/// Panics if the ffmpeg command could not be run.
///
/// # Errors
///
/// This function will return an error if the `Command spawn()` command returns an error.
pub fn add_text(
    input_file: &Path,
    text: &str,
    output_file: &Path,
) -> Result<Child> {
    let text = text.to_string().replace(':', "\\:");
    // TODO: Find a better way to handle fonts
    let vf_text = format!("drawtext='fontfile=C\\:/Windows/fonts/impact.ttf:fontcolor=white:borderw=3:fontsize=22:x=(w-text_w)/2:y=(h-text_h)-20:text={}'", text);
    Command::new("ffmpeg")
        .arg("-i")
        .arg(input_file)
        .args(["-vf", &vf_text])
        .arg("-y")
        .arg(output_file)
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.into())
}

/// Scales the `input_file` to the given `scale` (x, y) with the default value of `480x270`
/// using ffmpeg with codec copy `-c copy` meaning that this runs blazing fast™.
///
/// # Errors
///
/// This function will return an error if the `Command spawn()` command returns an error.
pub fn scale(
    input_file: &Path,
    scale: Option<(u32, u32)>,
    output_file: &Path,
) -> Result<Child> {
    let scale = scale.unwrap_or((480, 270));
    Command::new("ffmpeg")
        .arg("-i")
        .arg(input_file)
        .arg("-s")
        .arg(format!("{}x{}", scale.0, scale.1))
        .arg("-c")
        .arg("copy")
        .arg("-y")
        .arg(output_file)
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.into())
}

/// Converts the given file to a gif and and returns the handle to the spawned child process.
///
/// The output file will have the same name as the input but with the `.gif` file extension.
///
/// The ffmpeg command used looks like this:
///
/// ```powershell
/// ffmpeg -i <input_file> -y -filter_complex "[0:v] split [a][b];[a] palettegen [p];[b][p] paletteuse" <input_file.gif>
/// ```
///
/// # Errors
///
/// This function will return an error if there is an error spawning the child.
pub fn convert_to_gif(input_file: &Path) -> Result<Child> {
    let handle = Command::new("ffmpeg")
        .arg("-i")
        .arg(input_file)
        .arg("-y")
        .args([
            "-filter_complex",
            "[0:v] split [a][b];[a] palettegen [p];[b][p] paletteuse",
        ])
        .arg(&input_file.with_extension("gif"))
        .stderr(Stdio::null())
        .spawn()?;
    Ok(handle)
}

/// Stitches the `first_file` and the `second_file` together to create the `output_file`.
///
/// It uses the ffmpeg complex_filter "concat" with `unsafe=true` meaning that the resolution of
/// the two files doesn't need to match and the `second_file` gets stretched to the resolution
/// of the `first_file`.
///
/// For faster concatenation use the [`stitch_files_concat_demuxer()`] function.
///
/// # Errors
///
/// This function will return an error if there is a problem with spawning the command.
pub fn stitch_files(
    first_file: &Path,
    second_file: &Path,
    output_file: &Path,
) -> Result<()> {
    Command::new("ffmpeg")
        .arg("-i")
        .arg(first_file)
        .arg("-i")
        .arg(second_file)
        .args([
            "-filter_complex",
            "[0:v] [1:v] concat=n=2:v=1:unsafe=true [v]",
        ])
        .args(["-map", "[v]"])
        .arg(output_file)
        .arg("-y")
        // .stderr(Stdio::inherit())
        .output()?;
    Ok(())
}

/// Concatenates the two files using the ffmpeg concat demuxer -> faster than the concat
/// complex_filter used by [`stitch_files()`]
///
/// # Panics
///
/// Panics if the parent dir for the `concat_list.txt` can not be found.
///
/// # Errors
///
/// This function will return an error if there is a problem with creating the `concat_list.txt` file
/// or if the ffmpeg command cannot be run.
// Reference: https://stackoverflow.com/a/11175851/10018101
pub fn stitch_files_concat_demuxer(
    first_file: &Path,
    second_file: &Path,
    output_file: &Path,
) -> Result<()> {
    let file_list_text = format!(
        "file '{}'\nfile '{}'",
        first_file.as_os_str().to_string_lossy(),
        second_file.as_os_str().to_string_lossy()
    );

    let concat_list_path = temp_dir().join("time-for\\concat_list.txt");
    let parent_path = concat_list_path
        .parent()
        .ok_or(temp_dir().join("time-for").as_path())
        .expect("Get parent path");
    fs::create_dir_all(parent_path)?;
    let mut list_file = File::create(temp_dir().join("time-for\\concat_list.txt"))?;
    write!(list_file, "{}", file_list_text)?;

    Command::new("ffmpeg")
        .arg("-safe")
        .arg("0")
        .arg("-f")
        .arg("concat")
        .arg("-i")
        .arg(concat_list_path)
        .arg("-c")
        .arg("copy")
        .arg("-y")
        .arg(output_file)
        // .stderr(Stdio::inherit())
        .output()?;
    Ok(())
}

/// Check whether or not the `ffmpeg` command can be run
pub fn is_available() -> bool {
    Command::new("ffmpeg").output().is_ok()
}

#[derive(Error, Debug)]
pub enum FfmpegError {
    #[error("ffmpeg can not be found in path")]
    NotFound,
    #[error("Unknown Io error")]
    Io { source: io::Error },
}

impl From<io::Error> for FfmpegError {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => FfmpegError::NotFound,
            _ => FfmpegError::Io { source: err },
        }
    }
}

type Result<T> = std::result::Result<T, FfmpegError>;
