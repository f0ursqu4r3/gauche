use glam::UVec2;

pub enum SettingsMode {
    Main,
    Video,
    Audio,
    Controls,
}

pub struct Settings {
    pub mode: SettingsMode,

    pub video: VideoSettings,
    pub audio: AudioSettings,
    pub controls: ControlsSettings,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            mode: SettingsMode::Main,

            video: VideoSettings::new(),
            audio: AudioSettings::new(),
            controls: ControlsSettings::new(),
        }
    }
}

pub struct VideoSettings {
    pub resolution: UVec2,
    pub fullscreen: bool,
    pub vsync: bool,
    pub resolution_options: Vec<UVec2>,
}

impl VideoSettings {
    fn new() -> Self {
        Self {
            resolution: UVec2::new(1280, 720),
            fullscreen: false,
            vsync: false,
            resolution_options: vec![
                UVec2::new(800, 600),
                UVec2::new(1024, 768),
                UVec2::new(1280, 720),
                UVec2::new(1280, 1024),
                UVec2::new(1920, 1080),
            ],
        }
    }
}

pub struct AudioSettings {
    pub music_volume: f32,
    pub sfx_volume: f32,
}

impl AudioSettings {
    fn new() -> Self {
        Self {
            music_volume: 1.0,
            sfx_volume: 1.0,
        }
    }
}

pub struct ControlsSettings {
    pub jump: u32,
    pub shoot: u32,
}

impl ControlsSettings {
    fn new() -> Self {
        Self { jump: 0, shoot: 1 }
    }
}

pub const KEY_DEBOUNCE_INTERVAL: f32 = 0.2;
