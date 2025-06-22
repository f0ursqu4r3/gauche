use glam::*;
use rand::random_range;
use raylib::prelude::*;

pub const FRAMES_PER_SECOND: u32 = 60;
pub const TIMESTEP: f32 = 1.0 / FRAMES_PER_SECOND as f32;

use crate::{
    audio::{Audio, SoundEffect},
    entity::{Entity, EntityType, StepSound},
    entity_behavior::{
        move_entity_on_grid, pick_random_adjacent_tile_position_include_center, ready_to_move,
        reset_move_cooldown,
    },
    graphics::Graphics,
    particle::ParticleData,
    render::TILE_SIZE,
    sprite::Sprite,
    stage::{flip_stage_tiles, TileData},
    state::{Mode, State},
    tile::{self, can_build_on, is_tile_empty, is_tile_occupied},
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

    /* FYI: while loop makes step spin until catchup if we are behind some frames. This is on purpose*/
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

        state.particles.step();
        state.scene_frame = state.scene_frame.saturating_add(1);
    }

    if state.frame == u32::MAX {
        state.frame = 0;
    } else {
        state.frame += 1;
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
    // game over if no player
    if state.player_vid.is_none() {
        state.mode = Mode::GameOver;
        state.game_over = true;
        return;
    };

    // Player
    if let Some(player_vid) = state.player_vid {
        if ready_to_move(state, player_vid) {
            let player_tile_pos = state
                .entity_manager
                .get_entity(player_vid)
                .unwrap()
                .pos
                .as_ivec2();

            let wants_to_move_to = if state.playing_inputs.left {
                player_tile_pos + IVec2::new(-1, 0)
            } else if state.playing_inputs.right {
                player_tile_pos + IVec2::new(1, 0)
            } else if state.playing_inputs.up {
                player_tile_pos + IVec2::new(0, -1)
            } else if state.playing_inputs.down {
                player_tile_pos + IVec2::new(0, 1)
            } else {
                player_tile_pos // No movement input
            };

            if wants_to_move_to != player_tile_pos {
                move_entity_on_grid(
                    state,
                    audio,
                    player_vid,
                    wants_to_move_to,
                    false, // Do not ignore tile collision for player
                    false, // reset move cooldown
                );
            }
        }

        let target_cam_pos = state.entity_manager.get_entity(player_vid).unwrap().pos * TILE_SIZE;
        graphics.play_cam.pos = graphics.play_cam.pos.lerp(target_cam_pos, 0.1);
    }

    // --- Player Tile Logic ---
    if let Some(player_vid) = state.player_vid {
        let player = state.entity_manager.get_entity_mut(player_vid).unwrap();
        // check if place block input is pressed
        if state.playing_inputs.mouse_down.iter().any(|&down| down) {
            let player_grid_pos = player.pos.as_ivec2();
            let target_grid_pos = graphics.screen_to_tile(state.playing_inputs.mouse_pos);
            // Check if the target position is adjacent to the player
            let target_offset = target_grid_pos - player_grid_pos;
            pub const PLAYER_REACH: i32 = 2; // Player can reach 2 tiles away
            let in_range =
                target_offset.x.abs() <= PLAYER_REACH && target_offset.y.abs() <= PLAYER_REACH;
            if in_range {
                // if mouse 1 is pressed, place a block
                if state.playing_inputs.mouse_down[0] {
                    if can_build_on(state, target_grid_pos) {
                        // Place a block at the target position
                        state.stage.set_tile(
                            target_grid_pos.x as usize,
                            target_grid_pos.y as usize,
                            TileData {
                                tile: tile::Tile::Wall,
                                hp: 100,    // Example: full health for the block
                                variant: 0, // Example: default wall variant
                                flip_speed: 0,
                            },
                        );
                        audio.play_sound_effect(SoundEffect::BlockLand);
                    } else {
                        audio.play_sound_effect(SoundEffect::HitBlock1);
                    }
                    state.playing_inputs.mouse_down[0] = false;
                } else if state.playing_inputs.mouse_down[1] {
                    // If mouse 2 is pressed, remove a block
                    state.stage.set_tile(
                        target_grid_pos.x as usize,
                        target_grid_pos.y as usize,
                        TileData {
                            tile: tile::Tile::None,
                            hp: 0,      // Reset health to 0
                            variant: 0, // Reset to default empty tile
                            flip_speed: 0,
                        },
                    );
                    audio.play_sound_effect(SoundEffect::BlockLand);
                    state.playing_inputs.mouse_down[1] = false;
                }
            }
        }
    }

    // --- AI / Other Entity Logic ---
    let all_vids: Vec<_> = state.entity_manager.iter().map(|e| e.vid).collect();
    for vid in all_vids {
        if Some(vid) == state.player_vid || state.entity_manager.get_entity(vid).is_none() {
            continue;
        }

        if let Some(EntityType::Zombie) = state.entity_manager.get_entity(vid).map(|e| e.type_) {
            if ready_to_move(state, vid) {
                let current_tile_pos = state.entity_manager.get_entity(vid).unwrap().pos.as_ivec2();
                let wants_to_move_to =
                    pick_random_adjacent_tile_position_include_center(current_tile_pos);
                if wants_to_move_to != current_tile_pos {
                    move_entity_on_grid(
                        state,
                        audio,
                        vid,
                        wants_to_move_to,
                        false, // Do not ignore tile collision for zombies
                        false, // reset move cooldown
                    );
                }
            };
        }
    }

    // flip tile variants
    flip_stage_tiles(state);
}

/// Sets entity rotation from -15 to 15 degrees randomly
pub fn lean_entity(entity: &mut Entity) {
    entity.rot = random_range(-15.0..=15.0);
}

pub fn entity_step_sound_lookup(entity: &Entity) -> SoundEffect {
    // TODO: different step sounds based on entity type or state
    match entity.step_sound {
        StepSound::Step1 => SoundEffect::Step1,
        StepSound::Step2 => SoundEffect::Step2,
    }
}
