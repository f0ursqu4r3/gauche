pub struct Config {
    pub window_title: String,
    pub window_width: i32,
    pub window_height: i32,
}

impl Config {
    pub fn load(file_path: &str) -> Result<Config, String> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let config: toml::Value =
            toml::from_str(&content).map_err(|e| format!("Failed to parse config file: {}", e))?;

        Ok(Config {
            window_title: config
                .get("window_title")
                .and_then(toml::Value::as_str)
                .unwrap_or("Default Title")
                .to_string(),
            window_width: config
                .get("window_width")
                .and_then(toml::Value::as_integer)
                .unwrap_or(800) as i32,
            window_height: config
                .get("window_height")
                .and_then(toml::Value::as_integer)
                .unwrap_or(600) as i32,
        })
    }
}
