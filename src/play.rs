// this file contains functions to play the videos

extern crate ffmpeg_next as ffmpeg;

use std::{thread, time};

use ffmpeg::format::Pixel;
use ffmpeg::software::scaling::{context::Context, flag::Flags};
use ffmpeg::util::frame::video::Video;

use crate::config::Config;

const ASCII:[char;256] = [' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ','`','`','`','`','`','`','`','`','`','`','`','`','`','`','`','`','`','`','.','.','.','.','.','.','.','.','~','~','~','~','~','~','~','~','~','~','~','^','^','^','^','^','^','^','^','\'','\'','\'',',',',',',',',',',',',','|','|','_','_','_','_','_','_','_','_','/','/','/','/','/','/','/','*','+','+','<','>','>','>','>','}','[','{','7','=','=','=','=','=',';',';',';',';','i','i','l','l','?','?','?','?','1','1','r','t','t','c','v','v','v','I','"','"','"','"','"','"','u','u','J','3','3','3','3','3','3','3','3','3','3','C','C','C','C','2','5','y','4','4','4','0','e','e','e','e','x','x','x','x','O','O','k','k','$','w','Z','S','S','P','P','P','F','G','G','G','8','b','b','D','D','D','m','m','m','g','A','A','A','A','A','X','@','q','q','q','q','q','q','q','q','K','K','R','#','#','#','E','Q','Q','Q','Q','Q','Q','Q','Q','Q','Q','B','B','B','B','B','B','B','B','B','B','B','B','B','B','B','B','B','B','B','B','B','B','W','W','W','W','W','W','W','M'];

pub fn play_videos(config: Config) {
    ffmpeg::init().unwrap();
    print!("\x1b[?1049h");
    for file in config.files {

        loop {
            if let Ok(mut ictx) = ffmpeg::format::input(&file) {
                let input = ictx.streams().best(ffmpeg::media::Type::Video).expect("Could not find a stream");
                let video_stream_index = input.index();

                let frame_delay = time::Duration::from_micros((input.duration() as f64 * f64::from(input.time_base()) * 1_000_000.0 / input.frames() as f64) as u64 );

                let mut decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters()).expect("Could not create the context decoder")
                    .decoder().video().expect("Could not create the decoder");

                let mut scaler = Context::get( decoder.format(), decoder.width(), decoder.height(), Pixel::RGB24, 160, 90, Flags::POINT).unwrap();
                let mut last_scale=(0, 0);

                for (stream, packet) in ictx.packets() {
                    if stream.index() == video_stream_index {
                        let time = time::Instant::now();

                        decoder.send_packet(&packet).unwrap();

                        let mut decoded = Video::empty();

                        if decoder.receive_frame(&mut decoded).is_ok() {
                            let (mut w, h) = term_size::dimensions().unwrap();
                            w &= 0b11111111_11111111_11111111_11100000; // the scaler messes up if width is not a multiple of 32
                            if (w, h) != last_scale {
                                scaler = Context::get( decoder.format(), decoder.width(), decoder.height(), Pixel::RGB24, w as u32, h as u32, Flags::POINT).unwrap();
                                last_scale = (w, h);
                            }
                            let mut frame = Video::empty();
                            scaler.run(&decoded, &mut frame).expect("Failed to run the scaler");
                            (config.color_function)(&frame, w, h, config.buffer_size);
                        }

                        thread::sleep(frame_delay.saturating_sub(time::Instant::now().duration_since(time)));
                    }
                }
                decoder.send_eof().unwrap();
                if config.doloop == false {break; }
            }
        }
    }
    print!("\x1b[?1049l");
}

// use ' ' or 'M' characters to display a black and white frame
pub fn display_frame_bw(frame: &Video, width: usize, height: usize, buffer_size: usize) {
    let mut framebuffer = String::with_capacity(buffer_size);

    for y in 0..height {
        for x in 0..width {
            let brightness = ( ( frame.data(0)[3*(width*y + x) + 0] as f32) * 0.299 + ( frame.data(0)[3*(width*y + x) + 1] as f32) * 0.587 + ( frame.data(0)[3*(width*y + x) + 2] as f32) * 0.114 ) as u8;
            framebuffer.push( ASCII[if brightness < 128 {0} else {255}]);
        }
        framebuffer.push('\n');
    }
    framebuffer.pop();
    framebuffer.pop();
    print!("\x1b[;H{}", framebuffer);
}

// use ASCII characters to display a grayscale frame
pub fn display_frame_a(frame: &Video, width: usize, height: usize, buffer_size: usize) {
    let mut framebuffer = String::with_capacity(buffer_size);

    for y in 0..height {
        for x in 0..width {
            let brightness = ( ( frame.data(0)[3*(width*y + x) + 0] as f32) * 0.299 + ( frame.data(0)[3*(width*y + x) + 1] as f32) * 0.587 + ( frame.data(0)[3*(width*y + x) + 2] as f32) * 0.114 ) as usize;
            framebuffer.push( ASCII[brightness]);
        }
        framebuffer.push('\n');
    }
    framebuffer.pop();
    framebuffer.pop();
    print!("\x1b[;H{}", framebuffer);
}

// use 216 colors (~8 bit)
pub fn display_frame_8(frame: &Video, width: usize, height: usize, buffer_size: usize) {
    let mut framebuffer = String::with_capacity(buffer_size);

    for y in 0..height {
        for x in 0..width {
            let color =
                ( frame.data(0)[3*(width*y + x) + 0] as f32 * 3.0 / 128.0 ) as u8 * 36 +
                ( frame.data(0)[3*(width*y + x) + 1] as f32 * 3.0 / 128.0 ) as u8 * 6  +
                ( frame.data(0)[3*(width*y + x) + 2] as f32 * 3.0 / 128.0 ) as u8 * 1  + 16;
            framebuffer.push_str("\x1b[48;5;");
            framebuffer.push_str(&color.to_string());
            framebuffer.push_str("m ");
        }
        framebuffer.push('\n');
    }
    framebuffer.pop();
    framebuffer.pop();
    print!("\x1b[;H{}", framebuffer);
}

// use 24 bit (RGB) colors
pub fn display_frame_24(frame: &Video, width: usize, height: usize, buffer_size: usize) {
    let mut framebuffer = String::with_capacity(buffer_size);

    for y in 0..height {
        for x in 0..width {
            framebuffer.push_str("\x1b[48;2;");
            framebuffer.push_str(&frame.data(0)[3*(width*y + x) + 0].to_string());
            framebuffer.push(';');
            framebuffer.push_str(&frame.data(0)[3*(width*y + x) + 1].to_string());
            framebuffer.push(';');
            framebuffer.push_str(&frame.data(0)[3*(width*y + x) + 2].to_string());
            framebuffer.push_str("m ");
        }
        framebuffer.push('\n');
    }
    framebuffer.pop();
    framebuffer.pop();
    print!("\x1b[;H{}", framebuffer);
}
