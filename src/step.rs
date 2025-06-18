use glam::*;
use raylib::prelude::*;

pub const FRAMES_PER_SECOND: u32 = 60;
pub const TIMESTEP: f32 = 1.0 / FRAMES_PER_SECOND as f32;

use crate::{
    audio::{Audio, Song, SoundEffect},
    graphics::Graphics,
    state::{Mode, State},
    tile,
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

fn step_title(state: &mut State, audio: &mut Audio) {
    if state.menu_inputs.confirm {
        state.mode = Mode::Playing;
        audio.play_song(Song::Playing);
        audio.play_sound_effect(SoundEffect::SuperConfirm);

        // NOTE: When transitioning to Playing, you must create the player entity
        // and assign its VID to `state.player_vid`. This logic would typically
        // live in a dedicated stage/level initialization function.
        // For example:
        // state.player_vid = Some(entity_manager::spawn_player(&mut state.entity_manager, start_pos));
    }
}

/// Handles tile-based gameplay logic by operating on entities.
fn step_playing(state: &mut State, audio: &mut Audio, graphics: &mut Graphics) {
    // --- Player Control Logic ---
    // First, we check if a player entity even exists.
    if let Some(player_vid) = state.player_vid {
        // Then, we try to get a mutable reference to that entity from the manager.
        // This `if let Some` pattern is crucial because the player might have been destroyed.
        if let Some(player) = state.entity_manager.get_entity_mut(player_vid) {
            // Tick down the entity's personal movement cooldown.
            if player.move_cooldown > 0.0 {
                player.move_cooldown -= TIMESTEP;
            }

            // The player can only move if their cooldown is finished.
            if player.move_cooldown <= 0.0 {
                let mut target_pos = player.pos;
                let mut moved = false;

                // Check directional input.
                if state.playing_inputs.left {
                    target_pos.x -= 1.0;
                    moved = true;
                } else if state.playing_inputs.right {
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
                    let tile_coords = target_pos.as_ivec2();
                    if let Some(tile) = state
                        .stage
                        .get_tile(tile_coords.x as usize, tile_coords.y as usize)
                    {
                        if tile::walkable(*tile) {
                            player.pos = target_pos; // Update the entity's position.
                            audio.play_sound_effect(SoundEffect::BallBounce1);
                        }
                    }

                    // Reset the entity's cooldown.
                    const MOVE_COOLDOWN_TIME: f32 = 0.12;
                    player.move_cooldown = MOVE_COOLDOWN_TIME;
                }
            }

            // --- Camera Follow Logic (uses the entity's position) ---
            const TILE_SIZE: f32 = 16.0;
            let target_cam_pos = player.pos * TILE_SIZE;
            graphics.play_cam.pos = graphics.play_cam.pos.lerp(target_cam_pos, 0.1);
        } else {
            // The player's VID exists, but the entity is gone (e.g., destroyed).
            // This is the place to trigger a game over.
            state.mode = Mode::GameOver;
        }
    }
    // If `state.player_vid` is `None`, we simply do nothing.

    // --- AI / Other Entity Logic would go here ---
    // You could iterate through all other active entities and run their logic.
    // for entity in state.entity_manager.iter_mut().filter(|e| e.active && Some(e.vid) != state.player_vid) {
    //     // run_zombie_ai(entity, &state.world);
    // }
}
