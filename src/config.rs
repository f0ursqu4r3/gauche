use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub window: Window,
    pub game: Game,
}

#[derive(Deserialize, Debug)]
pub struct Window {
    pub title: String,
    pub width: i32,
    pub height: i32,
}

#[derive(Deserialize, Debug)]
pub struct Game {
    pub fps: i32,
    pub tile_size: i32,
}
