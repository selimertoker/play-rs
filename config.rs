use ffmpeg_next::util::frame::video::Video;
use std::env;
use crate::play::{display_frame_a, display_frame_8, display_frame_24};

pub struct Config {
    pub color_function: fn(&Video, usize, usize, usize),
    pub files: Vec<String>,
    pub buffer_size: usize
}

pub fn parse_args() -> Config {
    let mut args = env::args();
    let mut configuration = Config{color_function: display_frame_a, files: Vec::new(), buffer_size: 1048576};
    if args.len() <= 1 { help(None); }
    else {
        args.next().unwrap();
        for argument in args {
            if argument.starts_with("-") {
                let mut argchars = argument.chars();
                match argchars.nth(1).unwrap_or_else(||{ help(Some("no argument given")); 'x' }) {
                    '1' => { configuration.color_function = display_frame_a;  }
                    '2' => { configuration.color_function = display_frame_8;  }
                    '3' => { configuration.color_function = display_frame_24; }
                    'b' => { configuration.buffer_size = 
                        argchars.collect::<String>().parse::<usize>().unwrap_or_else(|num|{ 
                            help(Some(&format!("Couldnt parse the number: {num}"))); 0 }) }
                    'h' => { help(None); }
                    x   => { help(Some(&format!("unknown argument: {x}"))); }
                }
            }
            else { configuration.files.push(argument); }
        }

    }
    if configuration.files.len() == 0 { help(Some("no files to play")) }
    configuration
}

pub fn help(error: Option<&str>) {
    println!("{}",
             error.unwrap_or(
                 "\
play-rs
\t Terminal based video player written in Rust
Usage: play-rs [options] file
Options:
\t -h : print this help message
\t -1 : dont use color, use ASCII characters for brightness (default)
\t -2 : use 8-bit colors
\t -3 : use 24-bit colors
\t -b<n>: use framebuffer of size n bytes (default 1048576, may decrease performance if too low)
    "
    )
            );
    std::process::exit(0);
}
