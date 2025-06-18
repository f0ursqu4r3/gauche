use glam::*;
use rand::random_range;
use raylib::prelude::*;

pub const FRAMES_PER_SECOND: u32 = 60;
pub const TIMESTEP: f32 = 1.0 / FRAMES_PER_SECOND as f32;

use crate::{
    audio::{Audio, SoundEffect},
    entity::EntityType,
    graphics::Graphics,
    state::{Mode, State},
    tile::{self, is_tile_occupied},
};

pub fn step(
    rl: &mut RaylibHandle,
    _rlt: &mut RaylibThread,
    state: &mut State,
    audio: &mut Audio,
    graphics: &mut Graphics,
    _dt: f32,
) {
    state.time_since_last_update += rl.get_frame_time();

    while state.time_since_last_update > TIMESTEP {
        state.time_since_last_update -= TIMESTEP;

        if state.frame_pause > 0 {
            state.frame_pause -= 1;
            return;
        }

        match state.mode {
            Mode::Title => step_title(state, audio),
            Mode::Playing => step_playing(state, audio, graphics),
            _ => {} // Other modes
        }

        state.scene_frame = state.scene_frame.saturating_add(1);
    }
}

fn step_title(state: &mut State, _audio: &mut Audio) {
    if state.menu_inputs.confirm {
        // NOTE: The actual transition now happens in `process_input_title`
        // to ensure `init_playing_state` is called.
        // This remains for potential future menu logic.
    }
}

/// Handles tile-based gameplay logic by operating on entities.
fn step_playing(state: &mut State, audio: &mut Audio, graphics: &mut Graphics) {
    // --- Player Control Logic ---
    if let Some(player_vid) = state.player_vid {
        let can_move;
        let player_pos;
        {
            let player = state.entity_manager.get_entity_mut(player_vid).unwrap();
            if player.move_cooldown > 0.0 {
                player.move_cooldown -= TIMESTEP;
            }
            can_move = player.move_cooldown <= 0.0;
            player_pos = player.pos;
        }

        if can_move {
            let mut target_pos = player_pos;
            let mut moved = false;
            if state.playing_inputs.left {
                target_pos.x -= 1.0;
                moved = true;
            }
            // ... (other inputs) ...
            else if state.playing_inputs.right {
                target_pos.x += 1.0;
                moved = true;
            } else if state.playing_inputs.up {
                target_pos.y -= 1.0;
                moved = true;
            } else if state.playing_inputs.down {
                target_pos.y += 1.0;
                moved = true;
            }

            if moved {
                let target_grid_pos = target_pos.as_ivec2();
                let terrain_is_walkable = state
                    .stage
                    .get_tile(target_grid_pos.x as usize, target_grid_pos.y as usize)
                    .map_or(false, |t| tile::walkable(*t));
                let tile_is_unoccupied = !is_tile_occupied(state, target_grid_pos);

                if terrain_is_walkable && tile_is_unoccupied {
                    let player = state.entity_manager.get_entity_mut(player_vid).unwrap();
                    let old_grid_pos = player.pos.as_ivec2();
                    player.pos = target_pos;
                    audio.play_sound_effect(SoundEffect::BallBounce1);

                    // REFACTOR: Use the new, single-line interface for moving an entity.
                    state.move_entity_in_grid(player_vid, old_grid_pos, target_grid_pos);
                }

                let player = state.entity_manager.get_entity_mut(player_vid).unwrap();
                const MOVE_COOLDOWN_TIME: f32 = 0.12;
                player.move_cooldown = MOVE_COOLDOWN_TIME;
            }
        }

        let target_cam_pos = state.entity_manager.get_entity(player_vid).unwrap().pos * 16.0;
        graphics.play_cam.pos = graphics.play_cam.pos.lerp(target_cam_pos, 0.1);
    } else {
        state.mode = Mode::GameOver;
    }

    // --- AI / Other Entity Logic ---
    let all_vids: Vec<_> = state.entity_manager.iter().map(|e| e.vid).collect();
    for vid in all_vids {
        if Some(vid) == state.player_vid || state.entity_manager.get_entity(vid).is_none() {
            continue;
        }

        if let Some(EntityType::Zombie) = state.entity_manager.get_entity(vid).map(|e| e.type_) {
            let can_move;
            let zombie_pos;
            {
                let zombie = state.entity_manager.get_entity_mut(vid).unwrap();
                if zombie.move_cooldown > 0.0 {
                    zombie.move_cooldown -= TIMESTEP;
                }
                can_move = zombie.move_cooldown <= 0.0;
                zombie_pos = zombie.pos;
            }

            if can_move {
                let mut target_pos = zombie_pos;
                let direction = random_range(0..5);
                let moved = match direction {
                    0 => {
                        target_pos.x -= 1.0;
                        true
                    }
                    1 => {
                        target_pos.x += 1.0;
                        true
                    }
                    2 => {
                        target_pos.y -= 1.0;
                        true
                    }
                    3 => {
                        target_pos.y += 1.0;
                        true
                    }
                    _ => false,
                };

                if moved {
                    let target_grid_pos = target_pos.as_ivec2();
                    let terrain_is_walkable = state
                        .stage
                        .get_tile(target_grid_pos.x as usize, target_grid_pos.y as usize)
                        .map_or(false, |t| tile::walkable(*t));
                    let mut tile_is_unoccupied = true;
                    if terrain_is_walkable {
                        let entities_in_cell = &state.spatial_grid[target_grid_pos.x as usize]
                            [target_grid_pos.y as usize];
                        tile_is_unoccupied = !entities_in_cell.iter().any(|other_vid| {
                            if *other_vid == vid {
                                return false;
                            }
                            state
                                .entity_manager
                                .get_entity(*other_vid)
                                .map_or(false, |e| e.impassable)
                        });
                    }

                    if terrain_is_walkable && tile_is_unoccupied {
                        let zombie = state.entity_manager.get_entity_mut(vid).unwrap();
                        let old_grid_pos = zombie.pos.as_ivec2();
                        zombie.pos = target_pos;

                        // REFACTOR: Use the new interface for zombies too.
                        state.move_entity_in_grid(vid, old_grid_pos, target_grid_pos);
                    }
                }

                let zombie = state.entity_manager.get_entity_mut(vid).unwrap();
                const ZOMBIE_MOVE_COOLDOWN: f32 = 0.8;
                zombie.move_cooldown = ZOMBIE_MOVE_COOLDOWN;
            }
        }
    }
}
