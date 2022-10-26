use std::{fs, process::Command};

fn main() {
    println!("YES OW TIME");

    // Create text for gif
    let text = chrono::Local::now().format("It is %H\\:%M\\:%S %A %B %d %Y");
    let vf_text = format!("drawtext=fontfile=assets/Montserrat-Bold.ttf:fontcolor=white:borderw=3:fontsize=24:x=(w-text_w)/2:y=(h-text_h)-20:text='{}'", text);

    println!("Creating GIF...");
    Command::new("ffmpeg")
        .args(["-i", "assets/look_at_time.gif"])
        .args(["-vf", vf_text.as_str()])
        .arg("-y")
        .arg("assets/ow-time.gif")
        // .stdout(Stdio::inherit())
        // .stderr(Stdio::inherit())
        .output()
        .expect("error ffmpeg run");

    // Open output folder in windows explorer
    Command::new("explorer").arg("assets").output().unwrap();

    // Print path to output file
    let can_path = fs::canonicalize("assets/ow-time.gif")
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap();
    println!("{}", &can_path[4..]);

    println!("DONE DONE DONE DONE DONE DONE");
}
