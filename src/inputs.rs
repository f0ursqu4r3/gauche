use glam::*;
use raylib::prelude::*;

use crate::{
    audio::Audio,
    graphics::Graphics,
    settings::INVENTORY_SELECTION_DEBOUNCE_INTERVAL,
    stage::init_playing_state,
    state::{Mode, State},
};

pub fn process_input(
    rl: &mut RaylibHandle,
    rlt: &mut RaylibThread,
    state: &mut State,
    audio: &mut Audio,
    graphics: &mut Graphics,
    dt: f32,
) {
    // always update mouse inputs
    set_mouse_inputs(rl, state, dt);

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
///
#[derive(Debug, Clone, Copy)]
pub struct MouseInputs {
    pub left: bool,
    pub right: bool,
    pub pos: IVec2,
    pub scroll: f32,
}

impl MouseInputs {
    pub fn new() -> MouseInputs {
        MouseInputs {
            left: false,
            right: false,
            pos: IVec2::ZERO,
            scroll: 0.0,
        }
    }
}

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

    pub inventory_prev: bool,
    pub inventory_next: bool,

    pub mouse_pos: Vec2,
    pub mouse_down: [bool; 2],
}
impl PlayingInputs {
    pub fn new() -> PlayingInputs {
        PlayingInputs {
            left: false,
            right: false,
            up: false,
            down: false,

            inventory_prev: false,
            inventory_next: false,

            mouse_pos: Vec2::new(0.0, 0.0),
            mouse_down: [false; 2],
        }
    }
}

////////////////////////    STATE INPUT STRUCT FILLING     ////////////////////////

pub fn set_mouse_inputs(rl: &mut RaylibHandle, state: &mut State, _dt: f32) {
    let raw_mouse_pos = rl.get_mouse_position();
    let mouse_pos = IVec2::new(raw_mouse_pos.x as i32, raw_mouse_pos.y as i32);
    let scroll = rl.get_mouse_wheel_move();

    state.mouse_inputs = MouseInputs {
        left: rl.is_mouse_button_down(raylib::consts::MouseButton::MOUSE_BUTTON_LEFT),
        right: rl.is_mouse_button_down(raylib::consts::MouseButton::MOUSE_BUTTON_RIGHT),
        pos: mouse_pos,
        scroll,
    };
}

pub fn set_menu_inputs(rl: &mut RaylibHandle, state: &mut State, dt: f32) {
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
    state.menu_inputs = new_inputs;
}

pub fn set_playing_inputs(rl: &mut RaylibHandle, state: &mut State, dt: f32) {
    let mut new_inputs = PlayingInputs::new();
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
    new_inputs.inventory_prev = rl.is_key_down(raylib::consts::KeyboardKey::KEY_LEFT_BRACKET)
        || rl.is_gamepad_button_down(
            0,
            raylib::consts::GamepadButton::GAMEPAD_BUTTON_LEFT_TRIGGER_1,
        );
    new_inputs.inventory_next = rl.is_key_down(raylib::consts::KeyboardKey::KEY_RIGHT_BRACKET)
        || rl.is_gamepad_button_down(
            0,
            raylib::consts::GamepadButton::GAMEPAD_BUTTON_RIGHT_TRIGGER_1,
        );
    new_inputs.mouse_down[0] =
        rl.is_mouse_button_down(raylib::consts::MouseButton::MOUSE_BUTTON_LEFT);
    new_inputs.mouse_down[1] =
        rl.is_mouse_button_down(raylib::consts::MouseButton::MOUSE_BUTTON_RIGHT);

    // if rl.is_gamepad_available(0) {

    let raw_mouse_pos = rl.get_mouse_position();
    new_inputs.mouse_pos = Vec2::new(raw_mouse_pos.x, raw_mouse_pos.y);

    // debounce
    state.playing_input_debounce_timers.step(dt);
    new_inputs = state.playing_input_debounce_timers.debounce(&new_inputs);
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

    // if i hit space set the player .shake to 1.0
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_SPACE) {
        if let Some(player_vid) = state.player_vid {
            if let Some(player) = state.entity_manager.get_entity_mut(player_vid) {
                player.shake += 0.1;
            }
        }
    }

    // inventory management
    if let Some(player_vid) = state.player_vid {
        let player = state.entity_manager.get_entity_mut(player_vid).unwrap();
        if state.playing_inputs.inventory_next {
            player.inventory.increment_selected_index();
            state.playing_input_debounce_timers.inventory_next =
                INVENTORY_SELECTION_DEBOUNCE_INTERVAL;
        } else if state.playing_inputs.inventory_prev {
            player.inventory.decrement_selected_index();
            state.playing_input_debounce_timers.inventory_prev =
                INVENTORY_SELECTION_DEBOUNCE_INTERVAL;
        }
    }
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
            left: self.left == 0.0 && menu_inputs.left,
            right: self.right == 0.0 && menu_inputs.right,
            up: self.up == 0.0 && menu_inputs.up,
            down: self.down == 0.0 && menu_inputs.down,
            confirm: menu_inputs.confirm,
            back: menu_inputs.back,
        }
    }
}

pub struct PlayingInputDebounceTimers {
    pub inventory_prev: f32,
    pub inventory_next: f32,
}

impl PlayingInputDebounceTimers {
    pub fn new() -> PlayingInputDebounceTimers {
        PlayingInputDebounceTimers {
            inventory_prev: 0.0,
            inventory_next: 0.0,
        }
    }

    pub fn step(&mut self, dt: f32) {
        self.inventory_prev = (self.inventory_prev - dt).max(0.0);
        self.inventory_next = (self.inventory_next - dt).max(0.0);
    }

    pub fn debounce(&self, playing_inputs: &PlayingInputs) -> PlayingInputs {
        PlayingInputs {
            left: playing_inputs.left,
            right: playing_inputs.right,
            up: playing_inputs.up,
            down: playing_inputs.down,
            inventory_prev: self.inventory_prev == 0.0 && playing_inputs.inventory_prev,
            inventory_next: self.inventory_next == 0.0 && playing_inputs.inventory_next,
            mouse_pos: playing_inputs.mouse_pos,
            mouse_down: playing_inputs.mouse_down,
        }
    }
}
