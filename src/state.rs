use glam::Vec2;

use crate::{
    entity::VID,
    entity_manager::EntityManager,
    inputs::{MenuInputDebounceTimers, MenuInputs, PlayingInputs},
    stage::Stage,
    tile::Tile,
};

pub enum Mode {
    Title,
    Settings,
    VideoSettings,
    Playing,
    GameOver,
    Win,
}

pub struct State {
    pub mode: Mode,

    pub menu_inputs: MenuInputs,
    pub menu_input_debounce_timers: MenuInputDebounceTimers,
    pub playing_inputs: PlayingInputs,

    pub running: bool,
    pub now: f64,
    pub time_since_last_update: f32,
    pub scene_frame: u32,
    pub frame: u32,
    pub stage_frame: u32,

    pub game_over: bool,
    pub pause: bool,
    pub win: bool,

    pub points: u32,
    pub deaths: u32,
    pub frame_pause: u32,

    pub entity_manager: EntityManager,
    pub player_vid: Option<VID>,
    // pub special_effects: Vec<Box<dyn SpecialEffect>>,
    pub stage: Stage,

    pub rebuild_render_texture: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            mode: Mode::Title,
            menu_inputs: MenuInputs::new(),
            menu_input_debounce_timers: MenuInputDebounceTimers::new(),
            playing_inputs: PlayingInputs::new(),

            running: true,
            now: 0.0,
            time_since_last_update: 0.0,
            scene_frame: 0,
            frame: 0,
            stage_frame: 0,

            game_over: false,
            pause: false,
            win: false,

            points: 0,
            deaths: 0,
            frame_pause: 0,

            entity_manager: EntityManager::new(),
            player_vid: None,

            stage: Stage::new(crate::stage::StageType::TestArena, 64, 64),

            rebuild_render_texture: true,
        }
    }
}
