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
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_ESCAPE)
        || rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_Q)
    {
        state.running = false;
    }

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
        Mode::GameOver => process_input_game_over(rl, rlt, state, audio, graphics, dt),
        Mode::Win => {} //process_input_win(rl, rlt, state, audio, graphics, dt),
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

    pub num_row_1: bool,
    pub num_row_2: bool,
    pub num_row_3: bool,
    pub num_row_4: bool,
    pub num_row_5: bool,
    pub num_row_6: bool,
    pub num_row_7: bool,
    pub num_row_8: bool,
    pub num_row_9: bool,
    pub num_row_0: bool,

    pub arrow_left: bool,
    pub arrow_right: bool,
    pub arrow_up: bool,
    pub arrow_down: bool,
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

            num_row_1: false,
            num_row_2: false,
            num_row_3: false,
            num_row_4: false,
            num_row_5: false,
            num_row_6: false,
            num_row_7: false,
            num_row_8: false,
            num_row_9: false,
            num_row_0: false,

            arrow_left: false,
            arrow_right: false,
            arrow_up: false,
            arrow_down: false,
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
    // wasd
    {
        new_inputs.left = rl.is_key_down(raylib::consts::KeyboardKey::KEY_A);
        new_inputs.right = rl.is_key_down(raylib::consts::KeyboardKey::KEY_D);
        new_inputs.up = rl.is_key_down(raylib::consts::KeyboardKey::KEY_W);
        new_inputs.down = rl.is_key_down(raylib::consts::KeyboardKey::KEY_S);
    }

    // gamepad face keys
    {
        new_inputs.left |= rl.is_gamepad_button_down(
            0,
            raylib::consts::GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_LEFT,
        );
        new_inputs.right |= rl.is_gamepad_button_down(
            0,
            raylib::consts::GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_RIGHT,
        );
        new_inputs.up |= rl.is_gamepad_button_down(
            0,
            raylib::consts::GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_UP,
        );
        new_inputs.down |= rl.is_gamepad_button_down(
            0,
            raylib::consts::GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_DOWN,
        );
    }

    new_inputs.mouse_down[0] =
        rl.is_mouse_button_down(raylib::consts::MouseButton::MOUSE_BUTTON_LEFT);
    new_inputs.mouse_down[1] =
        rl.is_mouse_button_down(raylib::consts::MouseButton::MOUSE_BUTTON_RIGHT);

    // if rl.is_gamepad_available(0) {

    // num row inputs
    {
        new_inputs.num_row_1 = rl.is_key_down(raylib::consts::KeyboardKey::KEY_ONE);
        new_inputs.num_row_2 = rl.is_key_down(raylib::consts::KeyboardKey::KEY_TWO);
        new_inputs.num_row_3 = rl.is_key_down(raylib::consts::KeyboardKey::KEY_THREE);
        new_inputs.num_row_4 = rl.is_key_down(raylib::consts::KeyboardKey::KEY_FOUR);
        new_inputs.num_row_5 = rl.is_key_down(raylib::consts::KeyboardKey::KEY_FIVE);
        new_inputs.num_row_6 = rl.is_key_down(raylib::consts::KeyboardKey::KEY_SIX);
        new_inputs.num_row_7 = rl.is_key_down(raylib::consts::KeyboardKey::KEY_SEVEN);
        new_inputs.num_row_8 = rl.is_key_down(raylib::consts::KeyboardKey::KEY_EIGHT);
        new_inputs.num_row_9 = rl.is_key_down(raylib::consts::KeyboardKey::KEY_NINE);
        new_inputs.num_row_0 = rl.is_key_down(raylib::consts::KeyboardKey::KEY_ZERO);
    }

    // num row gamepad dpad 1-4 only
    {
        new_inputs.num_row_1 |= rl.is_gamepad_button_down(
            0,
            raylib::consts::GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_UP,
        );
        new_inputs.num_row_2 |= rl.is_gamepad_button_down(
            0,
            raylib::consts::GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_RIGHT,
        );
        new_inputs.num_row_3 |= rl.is_gamepad_button_down(
            0,
            raylib::consts::GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_DOWN,
        );
        new_inputs.num_row_4 |= rl.is_gamepad_button_down(
            0,
            raylib::consts::GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_LEFT,
        );
    }

    // arrow inputs
    {
        new_inputs.arrow_left = rl.is_key_down(raylib::consts::KeyboardKey::KEY_LEFT);
        new_inputs.arrow_right = rl.is_key_down(raylib::consts::KeyboardKey::KEY_RIGHT);
        new_inputs.arrow_up = rl.is_key_down(raylib::consts::KeyboardKey::KEY_UP);
        new_inputs.arrow_down = rl.is_key_down(raylib::consts::KeyboardKey::KEY_DOWN);
    }

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

    // if press num row keys, set inventory selected index
    if let Some(player_vid) = state.player_vid {
        if let Some(player) = state.entity_manager.get_entity_mut(player_vid) {
            if state.playing_inputs.num_row_1 {
                player.inventory.set_selected_index(0);
            } else if state.playing_inputs.num_row_2 {
                player.inventory.set_selected_index(1);
            } else if state.playing_inputs.num_row_3 {
                player.inventory.set_selected_index(2);
            } else if state.playing_inputs.num_row_4 {
                player.inventory.set_selected_index(3);
            } else if state.playing_inputs.num_row_5 {
                player.inventory.set_selected_index(4);
            } else if state.playing_inputs.num_row_6 {
                player.inventory.set_selected_index(5);
            } else if state.playing_inputs.num_row_7 {
                player.inventory.set_selected_index(6);
            } else if state.playing_inputs.num_row_8 {
                player.inventory.set_selected_index(7);
            } else if state.playing_inputs.num_row_9 {
                player.inventory.set_selected_index(8);
            } else if state.playing_inputs.num_row_0 {
                player.inventory.set_selected_index(9);
            }
        }
    }
}

// process input game over, on enter or space, go to title
pub fn process_input_game_over(
    rl: &mut RaylibHandle,
    _rlt: &mut RaylibThread,
    state: &mut State,
    _audio: &mut Audio,
    _graphics: &mut Graphics,
    _dt: f32,
) {
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_ENTER)
        || rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_SPACE)
        || rl.is_gamepad_button_pressed(
            0,
            raylib::consts::GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_DOWN,
        )
    {
        state.mode = Mode::Title;
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
            num_row_1: playing_inputs.num_row_1,
            num_row_2: playing_inputs.num_row_2,
            num_row_3: playing_inputs.num_row_3,
            num_row_4: playing_inputs.num_row_4,
            num_row_5: playing_inputs.num_row_5,
            num_row_6: playing_inputs.num_row_6,
            num_row_7: playing_inputs.num_row_7,
            num_row_8: playing_inputs.num_row_8,
            num_row_9: playing_inputs.num_row_9,
            num_row_0: playing_inputs.num_row_0,
            arrow_left: playing_inputs.arrow_left,
            arrow_right: playing_inputs.arrow_right,
            arrow_up: playing_inputs.arrow_up,
            arrow_down: playing_inputs.arrow_down,
        }
    }
}
