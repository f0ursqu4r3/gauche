/* This is the render mode dispatcher.
   Based on the state.mode, it will call the appropriate render function.
   Fiddly render logic should probably be elsewhere, since i expect a few different modes.
*/

use glam::Vec2;
use rand::random_range;
use raylib::prelude::*;

use crate::{
    entity::EntityType,
    graphics::Graphics,
    particle::{render_parallaxing_particles, render_particles, ParticleLayer},
    render_entities, render_tiles,
    render_ui::{
        draw_cursor, render_debug_info, render_hand_item, render_health_bar, render_inventory,
        render_item_range_indicator_base, render_item_range_indicator_top,
        render_selected_item_details,
    },
    state::{Mode, State},
    tile::get_tile_sprite,
};

pub const TILE_SIZE: f32 = 16.0;
pub const BACKGROUND_COLOR: Color = Color::new(10, 10, 10, 255);
pub const PLAY_AREA_BACKGROUND_COLOR: Color = Color::new(20, 20, 20, 255);

pub fn scale_and_blit_render_texture_to_window(
    draw_handle: &mut RaylibDrawHandle,
    graphics: &mut Graphics,
    render_texture: &mut RenderTexture2D,
) {
    // Blitting the render_texture to the screen
    let source_rec = Rectangle::new(
        0.0,
        0.0,
        render_texture.texture.width as f32,
        -render_texture.texture.height as f32,
    );
    // dest rec should be the fullscreen resolution if graphics.fullscreen, otherwise window_dims
    let dest_rec = if graphics.fullscreen {
        // get the fullscreen resolution
        let screen_width = draw_handle.get_screen_width();
        let screen_height = draw_handle.get_screen_height();
        Rectangle::new(0.0, 0.0, screen_width as f32, screen_height as f32)
    } else {
        Rectangle::new(
            0.0,
            0.0,
            graphics.window_dims.x as f32,
            graphics.window_dims.y as f32,
        )
    };

    let origin = Vector2::new(0.0, 0.0);

    // This is assuming the texture you want to draw is inside `render_texture.texture`
    draw_handle.draw_texture_pro(
        render_texture,
        source_rec,
        dest_rec,
        origin,
        graphics.camera.rotation,
        Color::WHITE,
    );
}

/// The main render dispatcher. It draws everything into an off-screen render texture
/// and then scales that texture to the window.
pub fn render(
    rl: &mut RaylibHandle,
    rlt: &mut RaylibThread,
    state: &mut State,
    graphics: &mut Graphics,
    render_texture: &mut RenderTexture2D,
) {
    // This is the primary handle for all drawing operations that happen on the final window.
    let mut draw_handle = rl.begin_drawing(rlt);
    {
        // We begin a texture mode, which redirects all subsequent drawing commands
        // to our off-screen render texture.
        let mut screen = draw_handle.begin_texture_mode(rlt, render_texture);
        screen.clear_background(BACKGROUND_COLOR);

        match state.mode {
            Mode::Title => render_title(state, graphics, &mut screen),
            Mode::Settings => render_settings_menu(state, graphics, &mut screen),
            Mode::VideoSettings => render_video_settings_menu(state, graphics, &mut screen),
            Mode::Playing => render_playing(state, graphics, &mut screen),
            Mode::GameOver => render_game_over(state, graphics, &mut screen),
            Mode::Win => render_win(state, graphics, &mut screen),
            // Add other states like StageTransition if they exist in the Mode enum
        }

        // draw cursor
        draw_cursor(state, &mut screen, graphics);
    } // The texture mode ends here automatically.

    // After drawing to the texture, we draw the texture itself to the screen.
    scale_and_blit_render_texture_to_window(&mut draw_handle, graphics, render_texture);
}

/// Renders a simple title screen.
pub fn render_title(
    _state: &mut State,
    graphics: &mut Graphics,
    screen: &mut RaylibTextureMode<RaylibDrawHandle>,
) {
    screen.clear_background(BACKGROUND_COLOR);

    let title = "GAUCHE";
    let font_size = 80;
    let text_width = screen.measure_text(title, font_size);
    screen.draw_text(
        title,
        (graphics.dims.x / 2) as i32 - (text_width / 2),
        (graphics.dims.y / 2) as i32 - 60,
        font_size,
        Color::WHITE,
    );

    let subtitle = "Press ENTER to Start";
    let sub_font_size = 22;
    let sub_text_width = screen.measure_text(subtitle, sub_font_size);
    screen.draw_text(
        subtitle,
        (graphics.dims.x / 2) as i32 - (sub_text_width / 2),
        (graphics.dims.y / 2) as i32 + 20,
        sub_font_size,
        Color::LIGHTGRAY,
    );
}

/// get alpha based on player view distance and position
/// so if player is at (0, 0) and view distance is 5, at 5 alpha will be 0, and next to player it will be 255
/// used for both tiles and player, so lets just have it take in two positions, root, and target
pub fn get_alpha_from_distance(root: Vec2, target: Vec2, view_distance: f32) -> u8 {
    let distance = (target - root).length();
    if distance >= view_distance {
        0 // Fully transparent
    } else {
        // Calculate alpha based on distance
        let alpha = ((1.0 - (distance / view_distance)) * 255.0) as u8;
        alpha.clamp(0, 255) // Ensure alpha is within valid range
    }
}

pub const VIEW_DISTANCE: f32 = 12.0 * TILE_SIZE;
/// wrapper for above that takes in state, and target
pub fn get_alpha_from_state(state: &State, target: Vec2) -> u8 {
    if let Some(player_vid) = state.player_vid {
        if let Some(player) = state.entity_manager.get_entity(player_vid) {
            get_alpha_from_distance(player.pos * TILE_SIZE, target, VIEW_DISTANCE)
        } else {
            0 // Player not found, return fully transparent
        }
    } else {
        0 // No player, return fully transparent
    }
}

/// Renders the main gameplay view.
pub fn render_playing(
    state: &mut State,
    graphics: &mut Graphics,
    screen: &mut RaylibTextureMode<RaylibDrawHandle>,
) {
    // --- Camera Setup ---
    graphics.camera.target = Vector2::new(graphics.play_cam.pos.x, graphics.play_cam.pos.y);
    graphics.camera.zoom = graphics.play_cam.zoom;
    let offset_vec = graphics.dims.as_vec2() / 2.0;
    graphics.camera.offset = Vector2::new(offset_vec.x, offset_vec.y);

    {
        let mut d = screen.begin_mode2D(graphics.camera);
        d.clear_background(BACKGROUND_COLOR);

        // --- World Rendering ---
        let world_width_pixels = state.stage.get_width() as f32 * TILE_SIZE;
        let world_height_pixels = state.stage.get_height() as f32 * TILE_SIZE;

        // Draw a background for the play area
        d.draw_rectangle(
            0,
            0,
            world_width_pixels as i32,
            world_height_pixels as i32,
            PLAY_AREA_BACKGROUND_COLOR,
        );

        // Get player position once to avoid re-fetching in the loop
        let player_pos_pixels = if let Some(player_vid) = state.player_vid {
            state
                .entity_manager
                .get_entity(player_vid)
                .map(|e| e.pos * TILE_SIZE)
        } else {
            None
        };

        render_tiles::render_tiles(&mut d, state, graphics, player_pos_pixels);

        render_particles(&mut d, state, graphics, ParticleLayer::Background);

        render_item_range_indicator_base(&mut d, state, graphics);

        render_entities::render_entities(&mut d, state, graphics, player_pos_pixels);

        render_particles(&mut d, state, graphics, ParticleLayer::Foreground);
        render_parallaxing_particles(&mut d, state, graphics);
        render_item_range_indicator_top(&mut d, state, graphics);
        render_hand_item(&mut d, state, graphics);
    }

    render_health_bar(state, graphics, screen);
    // render_debug_info(state, graphics, screen);

    // draw inventory
    render_inventory(state, graphics, screen);
    render_selected_item_details(state, graphics, screen);
}

// --- Stub Functions ---
pub fn render_settings_menu(
    _state: &mut State,
    _graphics: &mut Graphics,
    screen: &mut RaylibTextureMode<RaylibDrawHandle>,
) {
    screen.clear_background(Color::DARKGRAY);
    screen.draw_text("SETTINGS (STUB)", 20, 20, 30, Color::WHITE);
}
pub fn render_video_settings_menu(
    _state: &mut State,
    _graphics: &mut Graphics,
    screen: &mut RaylibTextureMode<RaylibDrawHandle>,
) {
    screen.clear_background(Color::DARKGRAY);
    screen.draw_text("VIDEO SETTINGS (STUB)", 20, 20, 30, Color::WHITE);
}
pub fn render_game_over(
    _state: &mut State,
    _graphics: &mut Graphics,
    screen: &mut RaylibTextureMode<RaylibDrawHandle>,
) {
    screen.clear_background(Color::MAROON);
    screen.draw_text("GAME OVER (STUB)", 20, 20, 30, Color::WHITE);
}
pub fn render_win(
    _state: &mut State,
    _graphics: &mut Graphics,
    screen: &mut RaylibTextureMode<RaylibDrawHandle>,
) {
    screen.clear_background(Color::GOLD);
    screen.draw_text("YOU WIN! (STUB)", 20, 20, 30, Color::WHITE);
}
