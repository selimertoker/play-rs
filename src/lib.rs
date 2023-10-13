pub mod config;
pub mod play;

pub fn help(message: Option<&str>) {
    println!("{}", message.unwrap_or(
"\
play-rs
\t Terminal based video player written in Rust
Usage: play-rs [options] file
Options:
\t -h : print this help message
\t -0 : black and white
\t -1 : grayscale, use ASCII characters for brightness (default)
\t -2 : use 8-bit colors
\t -3 : use 24-bit colors
\t -b<n>: use framebuffer of size n bytes (default 1048576, may decrease performance if too low)
\t -l : loop the video
"));
    std::process::exit(0);
}
