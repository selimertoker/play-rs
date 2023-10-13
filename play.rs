extern crate ffmpeg_next as ffmpeg;

use std::{thread, time};

use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};
use ffmpeg::util::frame::video::Video;

use crate::config::Config;

const ASCII:[char;256] = [' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ','`','`','`','`','`','`','`','`','`','`','`','`','`','`','`','`','`','`','.','.','.','.','.','.','.','.','~','~','~','~','~','~','~','~','~','~','~','^','^','^','^','^','^','^','^','\'','\'','\'',',',',',',',',',',',',','|','|','_','_','_','_','_','_','_','_','/','/','/','/','/','/','/','*','+','+','<','>','>','>','>','}','[','{','7','=','=','=','=','=',';',';',';',';','i','i','l','l','?','?','?','?','1','1','r','t','t','c','v','v','v','I','"','"','"','"','"','"','u','u','J','3','3','3','3','3','3','3','3','3','3','C','C','C','C','2','5','y','4','4','4','0','e','e','e','e','x','x','x','x','O','O','k','k','$','w','Z','S','S','P','P','P','F','G','G','G','8','b','b','D','D','D','m','m','m','g','A','A','A','A','A','X','@','q','q','q','q','q','q','q','q','K','K','R','#','#','#','E','Q','Q','Q','Q','Q','Q','Q','Q','Q','Q','B','B','B','B','B','B','B','B','B','B','B','B','B','B','B','B','B','B','B','B','B','B','W','W','W','W','W','W','W','M'];

pub fn play_videos(config: Config) {
    ffmpeg::init().unwrap();
    for file in config.files {

        if let Ok(mut ictx) = input(&file) {
            let input = ictx.streams().best(Type::Video).expect("Could not find a stream");
            let video_stream_index = input.index();

            let frame_delay = time::Duration::from_micros((input.duration() as f64 * f64::from(input.time_base()) * 1_000_000.0 / input.frames() as f64) as u64 );

            let context_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters()).expect("Could not create the context decoder");
            let mut decoder = context_decoder.decoder().video().expect("Could not create the decoder");

            let mut scaler = Context::get( decoder.format(), decoder.width(), decoder.height(), Pixel::RGB24, 160, 90, Flags::POINT).unwrap();
            let mut frame_index = 0;
            let mut last_scale=(160,90);

            let mut receive_and_process_decoded_frames = |decoder: &mut ffmpeg::decoder::Video| -> Result<(), ffmpeg::Error> {
                let mut decoded = Video::empty();
                while decoder.receive_frame(&mut decoded).is_ok() {
                    let (mut w, h) = term_size::dimensions().unwrap();
                    w &= 0b11111111_11111111_11111111_11100000; // the scaler messes up if width is not a multiple of 32
                    scaler = Context::get( decoder.format(), decoder.width(), decoder.height(), Pixel::RGB24, w as u32, h as u32, Flags::POINT).unwrap();
                    if (w, h) != last_scale { last_scale = (w, h); }
                    let mut rgb_frame = Video::empty();
                    scaler.run(&decoded, &mut rgb_frame).expect("Failed to run the scaler");
                    (config.color_function)(&rgb_frame, w, h, config.buffer_size);
                    frame_index += 1;
                }
                Ok(())
            };

            print!("\x1b[?1049h");
            for (stream, packet) in ictx.packets() {
                if stream.index() == video_stream_index {
                    let time = time::Instant::now();

                    decoder.send_packet(&packet).unwrap();
                    receive_and_process_decoded_frames(&mut decoder).unwrap();

                    thread::sleep(frame_delay.saturating_sub(time::Instant::now().duration_since(time)));
                }
            }
            decoder.send_eof().unwrap();
            receive_and_process_decoded_frames(&mut decoder).unwrap();
            print!("\x1b[?1049l");
        }
    }
}

pub fn display_frame_a(frame: &Video, width: usize, height: usize, buffer_size: usize) {
    let mut framebuffer = String::with_capacity(buffer_size);

    for y in 0..height {
        for x in 0..width {
            framebuffer.push( ASCII[(
                    ( frame.data(0)[3*(width*y + x) + 0] as f32) * 0.299 +
                    ( frame.data(0)[3*(width*y + x) + 1] as f32) * 0.587 +
                    ( frame.data(0)[3*(width*y + x) + 2] as f32) * 0.114 ) as usize ]);
        }
        framebuffer.push('\n');
    }
    framebuffer.pop();
    framebuffer.pop();
    print!("\x1b[;H{}", framebuffer);
}

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
