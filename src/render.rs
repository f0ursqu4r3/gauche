/* This is the render mode dispatcher.
   Based on the state.mode, it will call the appropriate render function.
   Fiddly render logic should probably be elsewhere, since i expect a few different modes.
*/

use raylib::prelude::*;

use crate::{
    graphics::Graphics,
    state::{Mode, State},
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

        // Draw Tiles and Grid
        for y in 0..state.stage.get_height() {
            for x in 0..state.stage.get_width() {
                d.draw_rectangle_lines_ex(
                    Rectangle::new(
                        x as f32 * TILE_SIZE,
                        y as f32 * TILE_SIZE,
                        TILE_SIZE,
                        TILE_SIZE,
                    ),
                    1.0,
                    Color::new(255, 255, 255, 2),
                );

                let tile_pixel_pos = Vector2::new(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE);

                // --- NEW: Draw Tile Sprites ---
                if let Some(tile) = state.stage.get_tile(x, y) {
                    // Match the tile type to its corresponding sprite enum
                    let maybe_sprite = match tile {
                        crate::tile::Tile::Grass => Some(crate::sprite::Sprite::Grass),
                        crate::tile::Tile::Wall => Some(crate::sprite::Sprite::Wall),
                        crate::tile::Tile::Ruin => Some(crate::sprite::Sprite::Ruin),
                        crate::tile::Tile::Water => Some(crate::sprite::Sprite::Water),
                        _ => None, // Tile::None has no sprite
                    };

                    // If a sprite is associated with the tile, draw it
                    if let Some(sprite) = maybe_sprite {
                        if let Some(texture) = graphics.get_sprite_texture(sprite) {
                            d.draw_texture(
                                texture,
                                tile_pixel_pos.x as i32,
                                tile_pixel_pos.y as i32,
                                Color::WHITE,
                            );
                        }
                    } else {
                        // For Tile::None, draw a black square to ensure it's empty
                        // d.draw_rectangle(
                        //     tile_pixel_pos.x as i32,
                        //     tile_pixel_pos.y as i32,
                        //     TILE_SIZE as i32,
                        //     TILE_SIZE as i32,
                        //     Color::BLACK,
                        // );
                    }
                }
                // --- End of new code ---

                // We can keep the faint grid overlay for debugging purposes
                // d.draw_rectangle_lines(
                //     tile_pixel_pos.x as i32,
                //     tile_pixel_pos.y as i32,
                //     TILE_SIZE as i32,
                //     TILE_SIZE as i32,
                //     Color::new(255, 255, 255, 30),
                // );
            }
        }

        // d.draw_rectangle_lines_ex(
        //     Rectangle::new(0.0, 0.0, world_width_pixels, world_height_pixels),
        //     2.0,
        //     Color::YELLOW,
        // );

        // --- Entity Rendering ---
        // (Entity rendering code remains unchanged)
        for entity in state.entity_manager.iter().filter(|e| e.active) {
            if let Some(texture) = graphics.get_sprite_texture(entity.sprite) {
                let entity_pixel_pos = entity.pos * TILE_SIZE;
                let source_rec =
                    Rectangle::new(0.0, 0.0, texture.width() as f32, texture.height() as f32);
                let dest_rec =
                    Rectangle::new(entity_pixel_pos.x, entity_pixel_pos.y, TILE_SIZE, TILE_SIZE);
                let origin = Vector2::new(TILE_SIZE / 2.0, TILE_SIZE / 2.0);

                d.draw_texture_pro(texture, source_rec, dest_rec, origin, 0.0, Color::WHITE);
            }
        }
    }

    // --- UI / Debug Text Rendering ---
    // (UI rendering code remains unchanged)
    let player_pos_text = if let Some(player_vid) = state.player_vid {
        if let Some(player) = state.entity_manager.get_entity(player_vid) {
            format!(
                "Player Pos: ({:.2}, {:.2})",
                player.pos.x as i32, player.pos.y as i32
            )
        } else {
            "Player: <DEAD>".to_string()
        }
    } else {
        "Player: <NONE>".to_string()
    };
    screen.draw_text(&player_pos_text, 10, 10, 20, Color::WHITE);
    let zoom_text = format!("Zoom: {:.2}x (Mouse Wheel)", graphics.play_cam.zoom);
    screen.draw_text(&zoom_text, 10, 35, 20, Color::WHITE);
    let entity_count = state.entity_manager.num_active_entities();
    let entity_text = format!("Active Entities: {}", entity_count);
    screen.draw_text(&entity_text, 10, 60, 20, Color::WHITE);
    let mouse_position = format!(
        "Mouse Pos: ({:.2}, {:.2})",
        state.playing_inputs.mouse_pos.x as u32, state.playing_inputs.mouse_pos.y as u32
    );
    screen.draw_text(&mouse_position, 10, 85, 20, Color::WHITE);
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
