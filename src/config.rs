use ffmpeg_next::util::frame::video::Video;
use std::env;
use crate::play::{display_frame_bw, display_frame_a, display_frame_8, display_frame_24};
use crate::help;

pub struct Config {
    pub color_function: fn(&Video, usize, usize, usize),
    pub files: Vec<String>,
    pub buffer_size: usize,
    pub doloop: bool,
}

pub fn parse_args() -> Config {
    let mut args = env::args();
    
    let mut config = Config{
        color_function: display_frame_a,
        files: Vec::new(),
        buffer_size: 1048576,
        doloop: false
    };
    
    if args.len() <= 1 { help(None); }
    else {
        args.next().unwrap();
        for argument in args {
            if argument.starts_with("-") {
                let mut argchars = argument.chars();
                match argchars.nth(1).unwrap_or_else(||{ help(Some("no argument given")); 'x' }) {
                    '0' => { config.color_function = display_frame_bw; }
                    '1' => { config.color_function = display_frame_a;  }
                    '2' => { config.color_function = display_frame_8;  }
                    '3' => { config.color_function = display_frame_24; }
                    'b' => { config.buffer_size = 
                        argchars.collect::<String>().parse::<usize>().unwrap_or_else(|num|{ 
                            help(Some(&format!("Couldnt parse the number: {num}"))); 0 }) }
                    'l' => { config.doloop = true;  }
                    'h' => { help(None); }
                    x   => { help(Some(&format!("unknown argument: {x}"))); }
                }
            }
            else { config.files.push(argument); }
        }

    }
    if config.files.len() == 0 { help(Some("no files to play")) }
    config
}
