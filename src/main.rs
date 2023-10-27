use ctrlc;

use play_rs::play::play_videos;
use play_rs::config::parse_args;

fn main() {
    ctrlc::set_handler(|| {print!("\x1b[?1049l\x1b[?25h"); std::process::exit(0)}).unwrap();
    play_videos(parse_args());
}

