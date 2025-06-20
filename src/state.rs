use glam::IVec2;

use crate::{
    entity::VID,
    entity_manager::EntityManager,
    inputs::{MenuInputDebounceTimers, MenuInputs, PlayingInputs},
    stage::Stage,
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

    pub spatial_grid: Vec<Vec<Vec<VID>>>,

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

            spatial_grid: vec![vec![vec![]; 64]; 64], // Adjust size as needed
            rebuild_render_texture: true,
        }
    }

    /// Adds an entity's VID to the spatial grid at a given position.
    pub fn add_entity_to_grid(&mut self, vid: VID, pos: IVec2) {
        if let Some(column) = self.spatial_grid.get_mut(pos.x as usize) {
            if let Some(cell) = column.get_mut(pos.y as usize) {
                cell.push(vid);
            }
        }
    }

    /// Removes an entity's VID from the spatial grid at a given position.
    pub fn remove_entity_from_grid(&mut self, vid: VID, pos: IVec2) {
        if let Some(column) = self.spatial_grid.get_mut(pos.x as usize) {
            if let Some(cell) = column.get_mut(pos.y as usize) {
                cell.retain(|v| *v != vid);
            }
        }
    }

    /// Moves an entity's VID from an old position to a new one in the spatial grid.
    pub fn move_entity_in_grid(&mut self, vid: VID, old_pos: IVec2, new_pos: IVec2) {
        self.remove_entity_from_grid(vid, old_pos);
        self.add_entity_to_grid(vid, new_pos);
    }

    /// Clears the spatial grid.
    pub fn clear_spatial_grid(&mut self) {
        for column in &mut self.spatial_grid {
            for cell in column {
                cell.clear();
            }
        }
    }

    /// Get all vids in rectangle defined by top-left and bottom-right corners.
    pub fn get_vids_in_rect(&self, top_left: IVec2, bottom_right: IVec2) -> Vec<VID> {
        let mut vids = Vec::new();
        for x in top_left.x..bottom_right.x {
            for y in top_left.y..bottom_right.y {
                if let Some(cell) = self
                    .spatial_grid
                    .get(x as usize)
                    .and_then(|col| col.get(y as usize))
                {
                    vids.extend_from_slice(cell);
                }
            }
        }
        vids
    }

    /// Get all vids in a rectangle defined by a center position and size.
    pub fn get_vids_in_rect_centered(&self, center: IVec2, size: IVec2) -> Vec<VID> {
        let half_size = size / 2;
        let top_left = center - half_size;
        let bottom_right = center + half_size;
        self.get_vids_in_rect(top_left, bottom_right)
    }
}
