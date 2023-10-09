extern crate ffmpeg_next as ffmpeg;

use std::{thread, time, env, io};
use std::io::Write;
use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};
use ffmpeg::util::frame::video::Video;
use ctrlc;

// const ASCII:[char; 256] = [' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ',' ','`','`','`','`','`','`','`','`','`','`','`','`','`','`','`','`','`','`','.','.','.','.','.','.','.','.','~','~','~','~','~','~','~','~','~','~','~','^','^','^','^','^','^','^','^','\'','\'','\'',',',',',',',',',',',',','|','|','_','_','_','_','_','_','_','_','/','/','/','/','/','/','/','*','+','+','<','>','>','>','>','}','[','{','7','=','=','=','=','=',';',';',';',';','i','i','l','l','?','?','?','?','1','1','r','t','t','c','v','v','v','I','"','"','"','"','"','"','u','u','J','3','3','3','3','3','3','3','3','3','3','C','C','C','C','2','5','y','4','4','4','0','e','e','e','e','x','x','x','x','O','O','k','k','$','w','Z','S','S','P','P','P','F','G','G','G','8','b','b','D','D','D','m','m','m','g','A','A','A','A','A','X','@','q','q','q','q','q','q','q','q','K','K','R','#','#','#','E','Q','Q','Q','Q','Q','Q','Q','Q','Q','Q','B','B','B','B','B','B','B','B','B','B','B','B','B','B','B','B','B','B','B','B','B','B','W','W','W','W','W','W','W','M'];

const ASCII:[u8; 256] = [32,32,32,32,32,32,32,32,32,32,32,32,32,32,32,32,32,32,32,32,96,96,96,96,96,96,96,96,96,96,96,96,96,96,96,96,96,96,46,46,46,46,46,46,46,46,126,126,126,126,126,126,126,126,126,126,126,94,94,94,94,94,94,94,94,39,39,39,44,44,44,44,44,44,124,124,95,95,95,95,95,95,95,95,47,47,47,47,47,47,47,42,43,43,60,62,62,62,62,125,91,123,55,61,61,61,61,61,59,59,59,59,105,105,108,108,63,63,63,63,49,49,114,116,116,99,118,118,118,73,34,34,34,34,34,34,117,117,74,51,51,51,51,51,51,51,51,51,51,67,67,67,67,50,53,121,52,52,52,48,101,101,101,101,120,120,120,120,79,79,107,107,36,119,90,83,83,80,80,80,70,71,71,71,56,98,98,68,68,68,109,109,109,103,65,65,65,65,65,88,64,113,113,113,113,113,113,113,113,75,75,82,35,35,35,69,81,81,81,81,81,81,81,81,81,81,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,87,87,87,87,87,87,87,77];

fn main() -> Result<(), ffmpeg::Error> {
    ctrlc::set_handler(|| {print!("\x1b[?1049l"); std::process::exit(0)}).unwrap();
    ffmpeg::init().unwrap();

    if let Ok(mut ictx) = input(&env::args().nth(1).expect("\x1b[31mGive file to play\x1b[0m")) {
        let input = ictx.streams().best(Type::Video).ok_or(ffmpeg::Error::StreamNotFound)?;
        let video_stream_index = input.index();

        let frame_delay = time::Duration::from_micros((input.duration() as f64 * f64::from(input.time_base()) * 1_000_000.0 / input.frames() as f64) as u64 );

        let context_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters())?;
        let mut decoder = context_decoder.decoder().video()?;

        let mut scaler = Context::get( decoder.format(), decoder.width(), decoder.height(), Pixel::RGB24, 160, 90, Flags::POINT)?;
        let mut frame_index = 0;
        let mut last_scale=(160,90);

        let mut receive_and_process_decoded_frames = |decoder: &mut ffmpeg::decoder::Video| -> Result<(), ffmpeg::Error> {
            let mut decoded = Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                let (mut w, h) = term_size::dimensions().unwrap();
                w &= 0b11111111_11111111_11111111_11100000; // the scaler messes up if width is not a multiple of 32
                scaler = Context::get( decoder.format(), decoder.width(), decoder.height(), Pixel::RGB24, w as u32, h as u32, Flags::POINT)?;
                if (w, h) != last_scale { last_scale = (w, h); }
                let mut rgb_frame = Video::empty();
                scaler.run(&decoded, &mut rgb_frame)?;
                display_image(&rgb_frame, w, h);
                frame_index += 1;
            }
            Ok(())
        };

        print!("\x1b[?1049h");
        for (stream, packet) in ictx.packets() {
            if stream.index() == video_stream_index {
                let time = time::Instant::now();

                decoder.send_packet(&packet)?;
                receive_and_process_decoded_frames(&mut decoder)?;

                thread::sleep(frame_delay.saturating_sub(time::Instant::now().duration_since(time)));
            }
        }
        decoder.send_eof()?;
        receive_and_process_decoded_frames(&mut decoder)?;
        print!("\x1b[?1049l");
    }

    Ok(())
}

fn display_image(frame: &Video, width: usize, height: usize) {
    let mut framebuffer:[u8; 1048576] = [b'\n'; 1048576];

    for y in 0..height {
        for x in 0..width {
            framebuffer[(width+1)*y + x] = ASCII[ (
                ( frame.data(0)[3*(width*y + x) + 0] as f32) * 0.299 +
                ( frame.data(0)[3*(width*y + x) + 1] as f32) * 0.587 +
                ( frame.data(0)[3*(width*y + x) + 2] as f32) * 0.114 ) as usize ] as u8;
        }
    }
    print!("\x1b[;H");
    io::stdout().write(&framebuffer[0..((width+1)*height-1)]).unwrap();
}
