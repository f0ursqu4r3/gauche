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

impl Config {
    pub fn load(file_path: &str) -> Result<Config, String> {
        // relative to src directory
        let path = std::path::Path::new("client/src").join(file_path);
        if !path.exists() {
            return Err(format!("Config file not found: {}", path.display()));
        }
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let config: Config =
            toml::from_str(&content).map_err(|e| format!("Failed to parse config file: {}", e))?;

        Ok(config)
    }
}
