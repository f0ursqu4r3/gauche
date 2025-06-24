use glam::IVec2;

use crate::{
    entity::VID,
    entity_manager::EntityManager,
    inputs::{
        MenuInputDebounceTimers, MenuInputs, MouseInputs, PlayingInputDebounceTimers, PlayingInputs,
    },
    particle::Particles,
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

    pub mouse_mode: bool,
    pub mouse_inputs: MouseInputs,

    pub menu_inputs: MenuInputs,
    pub menu_input_debounce_timers: MenuInputDebounceTimers,

    pub playing_inputs: PlayingInputs,
    pub playing_input_debounce_timers: PlayingInputDebounceTimers,

    pub running: bool,
    pub now: f64,
    pub time_since_last_update: f32,
    pub scene_frame: u32,
    pub frame: u32,

    pub game_over: bool,
    pub pause: bool,
    pub win: bool,

    pub points: u32,
    pub deaths: u32,
    pub frame_pause: u32,

    pub entity_manager: EntityManager,
    pub player_vid: Option<VID>,
    pub particles: Particles,
    pub stage: Stage,

    pub spatial_grid: Vec<Vec<Vec<VID>>>,

    pub rebuild_render_texture: bool,

    pub cloud_density: f32,
}

impl State {
    pub fn new() -> Self {
        Self {
            mode: Mode::Title,
            mouse_mode: true,
            mouse_inputs: MouseInputs::new(),
            menu_inputs: MenuInputs::new(),
            menu_input_debounce_timers: MenuInputDebounceTimers::new(),

            playing_inputs: PlayingInputs::new(),
            playing_input_debounce_timers: PlayingInputDebounceTimers::new(),

            running: true,
            now: 0.0,
            time_since_last_update: 0.0,
            scene_frame: 0,
            frame: 0,

            game_over: false,
            pause: false,
            win: false,

            points: 0,
            deaths: 0,
            frame_pause: 0,

            entity_manager: EntityManager::new(),
            player_vid: None,
            particles: Particles::new(),

            stage: Stage::new(crate::stage::StageType::TestArena, 64, 64),

            spatial_grid: vec![vec![vec![]; 64]; 64], // Adjust size as needed
            rebuild_render_texture: true,

            cloud_density: 0.5,
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

/// Helper to get all entities in adjacent tiles, not including center or diagonals.
/// This version is more direct and performs explicit bounds checking.
pub fn get_adjacent_entities(state: &State, pos: IVec2) -> Vec<VID> {
    let mut adjacent_entities = Vec::new();

    // Define the four cardinal directions to check. This is clearer than a nested loop.
    const OFFSETS: [IVec2; 4] = [
        IVec2::new(0, 1),  // Down
        IVec2::new(0, -1), // Up
        IVec2::new(1, 0),  // Right
        IVec2::new(-1, 0), // Left
    ];

    let grid_width = state.stage.get_width() as i32;
    let grid_height = state.stage.get_height() as i32;

    for offset in OFFSETS {
        let adjacent_pos = pos + offset;

        // Explicitly check if the position is within the grid's boundaries.
        // This is safer than relying on `.get()` to handle potential negative indices.
        if adjacent_pos.x >= 0
            && adjacent_pos.x < grid_width
            && adjacent_pos.y >= 0
            && adjacent_pos.y < grid_height
        {
            // We know the indices are valid, so we can safely access the grid.
            let cell = &state.spatial_grid[adjacent_pos.x as usize][adjacent_pos.y as usize];
            adjacent_entities.extend_from_slice(cell);
        }
    }

    adjacent_entities
}
