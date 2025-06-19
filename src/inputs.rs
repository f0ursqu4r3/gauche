use glam::*;
use raylib::prelude::*;

use crate::{
    audio::Audio,
    graphics::Graphics,
    settings::KEY_DEBOUNCE_INTERVAL,
    stage::init_playing_state,
    state::{Mode, State},
};

const JUMP_POWER: f32 = 10.0;

pub fn process_input(
    rl: &mut RaylibHandle,
    rlt: &mut RaylibThread,
    state: &mut State,
    audio: &mut Audio,
    graphics: &mut Graphics,
    dt: f32,
) {
    match state.mode {
        Mode::Playing => set_playing_inputs(rl, state, dt),
        _ => set_menu_inputs(rl, state, dt),
    }

    match state.mode {
        Mode::Title => process_input_title(rl, rlt, state, audio, graphics, dt),
        Mode::Settings => {} // process_input_settings_menu(rl, rlt, state, audio, graphics, dt),
        Mode::VideoSettings => {} //{process_input_video_settings_menu(rl, rlt, state, audio, graphics, dt)}
        Mode::Playing => process_input_playing(rl, rlt, state, audio, graphics, dt),
        Mode::GameOver => {} //process_input_game_over(rl, rlt, state, audio, graphics, dt),
        Mode::Win => {}      //process_input_win(rl, rlt, state, audio, graphics, dt),
    }
}

////////////////////////    INPUT DEFS    ////////////////////////

#[derive(Debug, Clone, Copy)]
pub struct MenuInputs {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,

    pub confirm: bool,
    pub back: bool,
}
impl MenuInputs {
    pub fn new() -> MenuInputs {
        MenuInputs {
            left: false,
            right: false,
            up: false,
            down: false,
            confirm: false,
            back: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PlayingInputs {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,

    pub mouse_pos: UVec2,
}
impl PlayingInputs {
    pub fn new() -> PlayingInputs {
        PlayingInputs {
            left: false,
            right: false,
            up: false,
            down: false,

            mouse_pos: UVec2::new(0, 0),
        }
    }
}

////////////////////////    STATE INPUT STRUCT FILLING     ////////////////////////

pub fn set_menu_inputs(rl: &mut RaylibHandle, state: &mut State, dt: f32) {
    let last_inputs = state.menu_inputs;
    let mut new_inputs = MenuInputs::new();

    new_inputs.left = rl.is_key_down(raylib::consts::KeyboardKey::KEY_LEFT)
        || rl.is_key_down(raylib::consts::KeyboardKey::KEY_A)
        || rl.is_gamepad_button_down(
            0,
            raylib::consts::GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_LEFT,
        );
    new_inputs.right = rl.is_key_down(raylib::consts::KeyboardKey::KEY_RIGHT)
        || rl.is_key_down(raylib::consts::KeyboardKey::KEY_D)
        || rl.is_gamepad_button_down(
            0,
            raylib::consts::GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_RIGHT,
        );
    new_inputs.up = rl.is_key_down(raylib::consts::KeyboardKey::KEY_UP)
        || rl.is_key_down(raylib::consts::KeyboardKey::KEY_W)
        || rl.is_gamepad_button_down(
            0,
            raylib::consts::GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_UP,
        );
    new_inputs.down = rl.is_key_down(raylib::consts::KeyboardKey::KEY_DOWN)
        || rl.is_key_down(raylib::consts::KeyboardKey::KEY_S)
        || rl.is_gamepad_button_down(
            0,
            raylib::consts::GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_DOWN,
        );

    new_inputs.confirm = rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_ENTER)
        || rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_SPACE)
        || rl.is_gamepad_button_pressed(
            0,
            raylib::consts::GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_DOWN,
        );
    new_inputs.back = rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_ESCAPE)
        || rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_BACKSPACE)
        || rl.is_gamepad_button_pressed(
            0,
            raylib::consts::GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_RIGHT,
        );

    // debounce
    state.menu_input_debounce_timers.step(dt);
    new_inputs = state.menu_input_debounce_timers.debounce(&new_inputs);
    state
        .menu_input_debounce_timers
        .reset_on_diff(&last_inputs, &new_inputs);
}

pub fn set_playing_inputs(rl: &mut RaylibHandle, state: &mut State, _dt: f32) {
    let kb_up = rl.is_key_down(raylib::consts::KeyboardKey::KEY_W);
    let kb_down = rl.is_key_down(raylib::consts::KeyboardKey::KEY_S);
    let kb_left = rl.is_key_down(raylib::consts::KeyboardKey::KEY_A);
    let kb_right = rl.is_key_down(raylib::consts::KeyboardKey::KEY_D);

    let mut gp_up = false;
    let mut gp_down = false;
    let mut gp_left = false;
    let mut gp_right = false;

    if rl.is_gamepad_available(0) {
        gp_up = rl.is_gamepad_button_down(
            0,
            raylib::consts::GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_UP,
        );
        gp_down = rl.is_gamepad_button_down(
            0,
            raylib::consts::GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_DOWN,
        );
        gp_left |= rl.is_gamepad_button_down(
            0,
            raylib::consts::GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_LEFT,
        );
        gp_right = rl.is_gamepad_button_down(
            0,
            raylib::consts::GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_RIGHT,
        );
    }

    let mut new_inputs = PlayingInputs::new();
    new_inputs.left = kb_left || gp_left;
    new_inputs.right = kb_right || gp_right;
    new_inputs.up = kb_up || gp_up;
    new_inputs.down = kb_down || gp_down;

    let raw_mouse_pos = rl.get_mouse_position();
    new_inputs.mouse_pos = UVec2::new(raw_mouse_pos.x as u32, raw_mouse_pos.y as u32);

    state.playing_inputs = new_inputs;
}

////////////////////////    PER GAME MODE INPUT PROCESSING     ////////////////////////

pub fn process_input_title(
    rl: &mut RaylibHandle,
    _rlt: &mut RaylibThread,
    state: &mut State,
    _audio: &mut Audio,
    graphics: &mut Graphics,
    _dt: f32,
) {
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_ENTER)
        || rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_SPACE)
        || rl.is_gamepad_button_pressed(
            0,
            raylib::consts::GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_DOWN,
        )
    {
        state.mode = Mode::Playing;
        init_playing_state(state, graphics);
    }
}

pub fn process_input_playing(
    rl: &mut RaylibHandle,
    _rlt: &mut RaylibThread,
    state: &mut State,
    _audio: &mut Audio,
    graphics: &mut Graphics,
    _dt: f32,
) {
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_ESCAPE)
        || rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_Q)
    {
        state.running = false;
    }

    // --- Camera Zoom Control (New!) ---
    let wheel_move = rl.get_mouse_wheel_move();
    if wheel_move != 0.0 {
        // Define zoom speed and limits
        const ZOOM_INCREMENT: f32 = 0.25;
        const MIN_ZOOM: f32 = 0.5;
        const MAX_ZOOM: f32 = 8.0;

        let wheel_direction = if wheel_move > 0.0 { 1.0 } else { -1.0 };

        // Adjust zoom based on wheel direction
        graphics.play_cam.zoom += wheel_direction * ZOOM_INCREMENT;

        // Clamp the zoom to the defined limits
        graphics.play_cam.zoom = graphics.play_cam.zoom.clamp(MIN_ZOOM, MAX_ZOOM);
    }

    // also do for - and = bc they are - and +
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_MINUS) {
        graphics.play_cam.zoom = (graphics.play_cam.zoom - 0.25).max(0.5);
    }

    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_EQUAL) {
        graphics.play_cam.zoom = (graphics.play_cam.zoom + 0.25).min(8.0);
    }

    // Mouse
    let raw_mouse_pos = rl.get_mouse_position();
    let mouse_tc = graphics.screen_tc(Vec2::new(raw_mouse_pos.x as f32, raw_mouse_pos.y as f32));
    state.playing_inputs.mouse_pos = UVec2::new(mouse_tc.x as u32, mouse_tc.y as u32);
    // println!("mouse pos {:?}", mouse_tc);
}

////////////////////////    INPUT DEBOUNCE TIMERS    ////////////////////////

/**
Does not debounce confirm and back inputs
*/
pub struct MenuInputDebounceTimers {
    pub left: f32,
    pub right: f32,
    pub up: f32,
    pub down: f32,
}

impl MenuInputDebounceTimers {
    pub fn new() -> MenuInputDebounceTimers {
        MenuInputDebounceTimers {
            left: 0.0,
            right: 0.0,
            up: 0.0,
            down: 0.0,
        }
    }

    pub fn step(&mut self, dt: f32) {
        self.left = (self.left - dt).max(0.0);
        self.right = (self.right - dt).max(0.0);
        self.up = (self.up - dt).max(0.0);
        self.down = (self.down - dt).max(0.0);
    }

    pub fn debounce(&self, menu_inputs: &MenuInputs) -> MenuInputs {
        MenuInputs {
            left: self.left > 0.0 && menu_inputs.left,
            right: self.right > 0.0 && menu_inputs.right,
            up: self.up > 0.0 && menu_inputs.up,
            down: self.down > 0.0 && menu_inputs.down,
            confirm: menu_inputs.confirm,
            back: menu_inputs.back,
        }
    }

    pub fn reset_on_diff(&mut self, last_inputs: &MenuInputs, new_inputs: &MenuInputs) {
        if new_inputs.left && !last_inputs.left {
            self.left = KEY_DEBOUNCE_INTERVAL;
        }
        if new_inputs.right && !last_inputs.right {
            self.right = KEY_DEBOUNCE_INTERVAL;
        }
        if new_inputs.up && !last_inputs.up {
            self.up = KEY_DEBOUNCE_INTERVAL;
        }
        if new_inputs.down && !last_inputs.down {
            self.down = KEY_DEBOUNCE_INTERVAL;
        }
    }
}
