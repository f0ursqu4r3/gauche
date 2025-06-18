use glam::*;
use raylib::prelude::*;

pub const FRAMES_PER_SECOND: u32 = 60;
pub const TIMESTEP: f32 = 1.0 / FRAMES_PER_SECOND as f32;

use crate::{
    audio::{Audio, SoundEffect},
    graphics::Graphics,
    state::{Mode, State},
};

pub fn step(
    rl: &mut RaylibHandle,
    rlt: &mut RaylibThread,
    state: &mut State,
    audio: &mut Audio,
    graphics: &mut Graphics,
    dt: f32,
) {
    let dt = rl.get_frame_time();
    state.time_since_last_update += dt;
    while state.time_since_last_update > TIMESTEP {
        state.time_since_last_update -= TIMESTEP;
        if state.frame_pause > 0 {
            state.frame_pause -= 1;
            return;
        }
        match state.mode {
            Mode::Title => step_title(rl, rlt, state, audio),
            Mode::Settings => {}
            Mode::VideoSettings => {}
            Mode::Playing => step_playing(rl, rlt, state, audio, graphics, dt),
            Mode::StageTransition => step_stage_transition(rl, rlt, state, audio, graphics),
            Mode::GameOver => step_game_over(rl, rlt, state, audio, graphics, dt),
            Mode::Win => step_win(rl, rlt, state, audio, graphics),
        }
        state.scene_frame = state.scene_frame.saturating_add(1);
    }
}

fn step_title(rl: &mut RaylibHandle, rlt: &mut RaylibThread, state: &mut State, audio: &mut Audio) {
    // Handle title screen logic here
    // For example, check for input to start the game
    if state.menu_inputs.confirm {
        state.mode = Mode::Playing;
        audio.play_sound_effect(SoundEffect::SuperConfirm);
    }
}

fn step_playing(
    rl: &mut RaylibHandle,
    rlt: &mut RaylibThread,
    state: &mut State,
    audio: &mut Audio,
    graphics: &mut Graphics,
    dt: f32,
) {
    // try to move player based on inputs
}

// // Update player position
// if move_player.x != 0.0 || move_player.y != 0.0 {
//     let new_pos = game.player.pos + move_player;
//     let tile = game.world.tile_at(new_pos.x as i32, new_pos.y as i32);
//     if let Some(tile) = tile {
//         if tile.walkable {
//             game.player.pos += move_player;
//         } else {
//             // Handle collision with non-walkable tiles
//             println!("Can't walk here! {:?}", tile);
//         }
//     }
// }
