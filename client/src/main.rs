mod config;

use config::Config;
use raylib::prelude::*;

fn main() {
    // Load configuration from a TOML file
    let config = Config::load("config.toml").expect("Failed to load configuration");

    let (mut rl, thread) = raylib::init()
        .size(config.window_width, config.window_height)
        .title(&config.window_title)
        .build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);
        d.draw_text("Hello, Raylib!", 190, 200, 30, Color::DARKGRAY);
    }
}
