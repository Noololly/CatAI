use tokio::process::Command;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use std::process::Stdio;

#[tokio::main]
async fn main() -> std::io::Result<()> {

    // Starts libcam as an asynchronous task
    let mut libcam = Command::new("libcamera-vid")
    .args(&["-o", "-", "--width", "640", "--height", "480", "--framerate", "30"])
    .stdout(Stdio::piped())
    .spawn()?;

    // Starts ffmpeg as an asyncronous task
    let mut ffmpeg = Command::new("ffmpeg")
    .args(&[
        "-y",
        "-f", "rawvideo",
        "-pix_fmt", "yuv420p", // Pixel format
        "-s", "640x480", // Frame size
        "-r", "30", // Frame rate
        "-i", "pipe:0", // Input from stdin
        "output.mp4", // Output file
    ]
    )
    .stdin(Stdio::piped())
    .spawn()?;


    let libcam_stdout = libcam.stdout.take().expect("Failed to capture stdout"); // get libcam's output
    let mut ffmpeg_stdin = ffmpeg.stdin.take().expect("Failed to open ffmpeg stdin"); // get ffmpeg's input

    let mut reader = BufReader::new(libcam_stdout); // reader for libcam's output
    let mut buffer = vec![0u8; 4096]; // a buffer to put libcam's input into

    loop {
        let bytes_read = reader.read(&mut buffer).await?; // reads output and puts it in buffer
        if bytes_read == 0 { // if the end, exit
            break;
        }

        ffmpeg_stdin.write_all(&buffer[..bytes_read]).await?; // give data to ffmpeg
    }

    drop(ffmpeg_stdin); // release ffmpegg's input

    // wait for both processes to end
    libcam.wait().await?;
    ffmpeg.wait().await?;


    Ok(())
}