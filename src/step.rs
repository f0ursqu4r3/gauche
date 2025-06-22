use glam::*;
use rand::random_range;
use raylib::prelude::*;

pub const FRAMES_PER_SECOND: u32 = 60;
pub const TIMESTEP: f32 = 1.0 / FRAMES_PER_SECOND as f32;

use crate::{
    audio::{Audio, SoundEffect},
    entity::{Entity, StepSound, VID},
    entity_behavior::{
        die_if_health_zero, growl_sometimes, indiscriminately_attack_nearby, move_entity_on_grid,
        ready_to_move, step_attack_cooldown, wander,
    },
    graphics::Graphics,
    render::TILE_SIZE,
    stage::{flip_stage_tiles, TileData},
    state::{Mode, State},
    tile::{self, can_build_on},
};

pub const PLACE_TILE_COOLDOWN: f32 = 0.05; // Cooldown for placing tiles in seconds

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

        if state.frame == u32::MAX {
            state.frame = 0;
        } else {
            state.frame += 1;
        }
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

    step_place_tile_cooldown(state);

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
    let ready_to_place = ready_to_place_tile(state);
    let mut reset_place_tile_cooldown = false;
    if let Some(player_vid) = state.player_vid {
        let player = state.entity_manager.get_entity_mut(player_vid).unwrap();
        // check if place block input is pressed
        if ready_to_place && (state.mouse_inputs.left || state.mouse_inputs.right) {
            let player_grid_pos = player.pos.as_ivec2();
            let target_grid_pos = graphics.screen_to_tile(state.mouse_inputs.pos.as_vec2());
            // Check if the target position is adjacent to the player
            let target_offset = target_grid_pos - player_grid_pos;
            pub const PLAYER_REACH: i32 = 2; // Player can reach 2 tiles away
            let in_range =
                target_offset.x.abs() <= PLAYER_REACH && target_offset.y.abs() <= PLAYER_REACH;
            if in_range {
                // if mouse 1 is pressed, place a block
                if state.mouse_inputs.left {
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
                        reset_place_tile_cooldown = true;
                    }
                } else if state.mouse_inputs.right {
                    // if the tile was a block, play sound effect
                    // and remove it
                    let tile_type = state
                        .stage
                        .get_tile_type(target_grid_pos.x as usize, target_grid_pos.y as usize);
                    if tile_type.is_some() && tile_type.unwrap() == tile::Tile::Wall {
                        // Play sound effect for removing a block
                        audio.play_sound_effect(SoundEffect::BlockLand);
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
                        reset_place_tile_cooldown = true;
                    }
                }
            }
            state.mouse_inputs.left = false; // Reset left mouse button state
            state.mouse_inputs.right = false; // Reset right mouse button state
        }
    }
    if reset_place_tile_cooldown {
        state.place_tile_cooldown_countdown = PLACE_TILE_COOLDOWN;
    }

    // --- AI / Other Entity Logic ---
    for vid in state.entity_manager.get_active_vids() {
        wander(state, audio, vid);
        entity_shake_attenuation(state, vid);
        growl_sometimes(state, audio, vid);
        indiscriminately_attack_nearby(state, audio, vid);
        die_if_health_zero(state, vid);
        step_attack_cooldown(state, vid);
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

pub fn entity_shake_attenuation(state: &mut State, vid: VID) {
    let entity = state.entity_manager.get_entity_mut(vid).unwrap();
    pub const SHAKE_ATTENUATION_RATE: f32 = 0.01;
    if entity.shake > SHAKE_ATTENUATION_RATE {
        entity.shake -= SHAKE_ATTENUATION_RATE
    } else {
        entity.shake = 0.0
    }
}

pub fn ready_to_place_tile(state: &State) -> bool {
    // Check if the cooldown is over
    if state.place_tile_cooldown_countdown <= 0.0 {
        return true;
    }
    false
}

pub fn step_place_tile_cooldown(state: &mut State) {
    if state.place_tile_cooldown_countdown > 0.0 {
        state.place_tile_cooldown_countdown -= TIMESTEP;
    } else {
        state.place_tile_cooldown_countdown = 0.0; // Reset to 0 when cooldown is over
    }
}
