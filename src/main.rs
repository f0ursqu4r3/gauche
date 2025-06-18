mod audio;
mod entity;
mod entity_manager;
mod graphics;
mod inputs;
mod render;
mod settings;
mod sprite;
mod state;
mod step;
mod tile;

use raylib::{audio::RaylibAudio, ffi::SetTraceLogLevel, prelude::TraceLogLevel};
use render::render;
use step::step;

use crate::inputs::process_input;
use crate::state::Mode;

fn main() {
    ////////////////        GRAPHICS INIT        ////////////////
    let (mut rl, mut rlt) = raylib::init().title("Gauche").build();
    unsafe {
        SetTraceLogLevel(TraceLogLevel::LOG_WARNING as i32);
    }
    let mut graphics = match graphics::Graphics::new(&mut rl, &rlt) {
        Ok(graphics) => graphics,
        Err(e) => {
            println!("Error initializing graphics: {}", e);
            std::process::exit(1);
        }
    };

    ////////////////        AUDIO INIT        ////////////////
    let rl_audio_device = match RaylibAudio::init_audio_device() {
        Ok(rl_audio_device) => rl_audio_device,
        Err(e) => {
            println!("Error initializing audio device: {}", e);
            std::process::exit(1);
        }
    };
    let mut audio = match audio::Audio::new(&rl_audio_device) {
        Ok(audio) => audio,
        Err(e) => {
            println!("Error initializing audio: {}", e);
            std::process::exit(1);
        }
    };
    audio.set_music_volume(1.0);
    audio.set_sfx_volume(1.0);
    // audio.play_song(Song::Title);

    ////////////////        MAIN LOOP        ////////////////
    let mut state = state::State::new();
    state.running = true;
    // DEBUG: this is temporary to auto jump into start
    // state.mode = Mode::Playing;
    let mut render_texture = match rl.load_render_texture(&rlt, graphics.dims.x, graphics.dims.y) {
        Ok(rt) => rt,
        Err(e) => {
            println!("Error creating render texture: {}", e);
            std::process::exit(1);
        }
    };

    while state.running && !rl.window_should_close() {
        // user may have changed internal res via video settings menu
        if state.rebuild_render_texture {
            render_texture = match rl.load_render_texture(&rlt, graphics.dims.x, graphics.dims.y) {
                Ok(rt) => rt,
                Err(e) => {
                    println!("Error creating render texture: {}", e);
                    std::process::exit(1);
                }
            };
            state.rebuild_render_texture = false;
        }

        // primary game loop process
        let dt = rl.get_frame_time();
        process_input(&mut rl, &mut rlt, &mut state, &mut audio, &mut graphics, dt);
        step(&mut rl, &mut rlt, &mut state, &mut audio, &mut graphics, dt);
        render(
            &mut rl,
            &mut rlt,
            &mut state,
            &mut graphics,
            &mut render_texture,
        );
        audio.update_current_song_stream_data();
    }

    while !rl.window_should_close() {}
}
