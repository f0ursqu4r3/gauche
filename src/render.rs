/* This is the render mode dispatcher.
   Based on the state.mode, it will call the appropriate render function.
   Fiddly render logic should probably be elsewhere, since i expect a few different modes.
*/

use std::vec;

use glam::Vec2;
use rand::random_range;
use raylib::prelude::*;

use crate::{
    entity::{self, EntityType},
    graphics::Graphics,
    particle::{render_particles, ParticleLayer},
    sprite::Sprite,
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

        // --- Draw Tiles ---
        for y in 0..state.stage.get_height() {
            'row: for x in 0..state.stage.get_width() {
                let tile_pixel_pos = Vec2::new(x as f32, y as f32) * TILE_SIZE;

                if let Some(tile_data) = state.stage.get_tile(x, y) {
                    let sprite = match get_tile_sprite(&tile_data) {
                        Some(s) => s,
                        None => continue 'row, // Skip if no sprite found
                    };

                    if let Some(texture) = graphics.get_sprite_texture(sprite) {
                        // Calculate alpha based on distance from player
                        let alpha = if let Some(player_pos) = player_pos_pixels {
                            let distance = (tile_pixel_pos - player_pos).length();
                            let tile_distance = (distance / TILE_SIZE).floor() as u32;
                            let max_steps = (VIEW_DISTANCE / TILE_SIZE) as u32;
                            let step_alpha = if tile_distance >= max_steps {
                                0
                            } else {
                                // 255 at center, 0 at max_steps
                                (((max_steps - tile_distance) as f32 / max_steps as f32) * 255.0)
                                    as u8
                            };
                            step_alpha
                        } else {
                            255 // If no player, everything is fully visible
                        };

                        // Only draw if it's visible at all
                        if alpha > 0 {
                            d.draw_texture(
                                texture,
                                tile_pixel_pos.x as i32,
                                tile_pixel_pos.y as i32,
                                Color::new(255, 255, 255, alpha),
                            );
                        }
                    }
                }
            }
        }

        render_particles(&mut d, state, graphics, ParticleLayer::Background);

        // --- Entity Rendering ---
        for entity in state.entity_manager.iter().filter(|e| e.active) {
            // Player is always fully visible
            let alpha = if entity.type_ == EntityType::Player {
                255
            } else if let Some(player_pos) = player_pos_pixels {
                // Other entities fade based on distance
                get_alpha_from_distance(player_pos, entity.pos * TILE_SIZE, VIEW_DISTANCE)
            } else {
                255 // If no player, everything is fully visible
            };

            // Only draw if visible
            if alpha > 0 {
                if let Some(texture) = graphics.get_sprite_texture(entity.sprite) {
                    let entity_pixel_pos = entity.pos * TILE_SIZE;
                    let source_rec =
                        Rectangle::new(0.0, 0.0, texture.width() as f32, texture.height() as f32);
                    // apply shake
                    let position = if entity.shake == 0.0 {
                        entity_pixel_pos
                    } else {
                        // Apply shake offset
                        let shake_offset = entity.shake * TILE_SIZE;
                        // random offset in x and y
                        let shake_x = random_range(-shake_offset..shake_offset);
                        let shake_y = random_range(-shake_offset..shake_offset);
                        // Apply shake to position
                        let shake_offset = Vec2::new(shake_x, shake_y);
                        Vec2::new(
                            entity_pixel_pos.x + shake_offset.x,
                            entity_pixel_pos.y + shake_offset.y,
                        )
                    };

                    let dest_rec = Rectangle::new(position.x, position.y, TILE_SIZE, TILE_SIZE);
                    let origin = Vector2::new(TILE_SIZE / 2.0, TILE_SIZE / 2.0);
                    d.draw_texture_pro(
                        texture,
                        source_rec,
                        dest_rec,
                        origin,
                        entity.rot,
                        Color::new(255, 255, 255, alpha),
                    );
                }
            }
        }

        render_particles(&mut d, state, graphics, ParticleLayer::Foreground);
    }

    render_health_bar(state, graphics, screen);

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
    let mouse_position = format!("Mouse Pos: ({:.2}, {:.2})", { state.mouse_inputs.pos.x }, {
        state.mouse_inputs.pos.y
    });
    screen.draw_text(&mouse_position, 10, 85, 20, Color::WHITE);

    // draw inventory
    render_inventory(state, graphics, screen);
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

/// Renders a stylized, offset, angled health bar for the player.
pub fn render_health_bar(
    state: &State,
    graphics: &Graphics,
    screen: &mut RaylibTextureMode<RaylibDrawHandle>,
) {
    // --- 1. Get Player's Health Percentage ---
    const MAX_HEALTH: f32 = 100.0;
    let mut health_percentage = 0.0; // Default to 75% for visualization

    if let Some(player_vid) = state.player_vid {
        if let Some(player) = state.entity_manager.get_entity(player_vid) {
            // Check if health is > 0 to avoid using the default visualization value
            if player.health > 0 {
                health_percentage = (player.health as f32 / MAX_HEALTH).clamp(0.0, 1.0);
            }
        }
    }

    // --- 2. Define Bar Geometry & Style ---
    let screen_width = graphics.dims.x as f32;
    let screen_height = graphics.dims.y as f32;

    const BACKGROUND_ANGLE: f32 = -2.0;
    const HEALTH_BAR_ANGLE: f32 = -3.0;

    let bar_width = screen_width * 0.25;
    let bar_height = 30.0;

    let container_pos = Vector2::new(
        screen_width * 0.05,
        screen_height - bar_height - (screen_height * 0.05),
    );

    // Define the offset for the red bar (e.g., 8 pixels right and 8 pixels up)
    const OFFSET_AMOUNT: f32 = 4.0;
    const RED_BAR_OFFSET: Vector2 = Vector2::new(OFFSET_AMOUNT, -OFFSET_AMOUNT);

    // The rotation origin for the background bar (its left-center edge)
    let background_origin = Vector2::new(0.0, bar_height / 2.0);

    // --- 3. Draw the Background Bar ---
    let background_rect = Rectangle::new(container_pos.x, container_pos.y, bar_width, bar_height);
    screen.draw_rectangle_pro(
        background_rect,
        background_origin,
        BACKGROUND_ANGLE,
        Color::new(10, 10, 10, 220),
    );

    // --- 4. Draw the Offset Red Health Bar ---
    if health_percentage > 0.0 {
        let health_fill_width = bar_width * health_percentage;

        // Apply the positional offset to the health bar's rectangle
        let health_rect = Rectangle::new(
            container_pos.x + RED_BAR_OFFSET.x,
            container_pos.y + RED_BAR_OFFSET.y,
            health_fill_width,
            bar_height,
        );

        // To make the offset bar rotate around the same world-space pivot as the background,
        // we must compensate its local origin for the positional offset.
        let compensated_origin = Vector2::new(
            background_origin.x - RED_BAR_OFFSET.x,
            background_origin.y - RED_BAR_OFFSET.y,
        );

        screen.draw_rectangle_pro(
            health_rect,
            compensated_origin, // Use the new, compensated origin
            HEALTH_BAR_ANGLE,
            Color::RED,
        );
    }
}

// #[derive(Debug, Clone, Copy)]
// pub struct MouseInputs {
//     pub left: bool,
//     pub right: bool,
//     pub position: IVec2,
//     pub scroll: f32,
// }
/// Draw a cursor at the mouse position.
/// use sprite Cursor
pub fn draw_cursor(
    state: &State,
    screen: &mut RaylibTextureMode<RaylibDrawHandle>,
    graphics: &Graphics,
) {
    let mouse_pos = state.mouse_inputs.pos.as_vec2();
    let cursor_texture = graphics.get_sprite_texture(Sprite::Cursor);
    if let Some(texture) = cursor_texture {
        // let cursor_size = Vec2::new(texture.width() as f32, texture.height() as f32);
        let cursor_pos = mouse_pos;
        screen.draw_texture(
            texture,
            cursor_pos.x as i32,
            cursor_pos.y as i32,
            Color::WHITE,
        );
    };
}

pub fn render_inventory(
    state: &State,
    _graphics: &Graphics,
    screen: &mut RaylibTextureMode<RaylibDrawHandle>,
) {
    if let Some(player_vid) = state.player_vid {
        if let Some(player) = state.entity_manager.get_entity(player_vid) {
            let mut inv_slots = vec![None; 10];
            for (i, item) in player.inventory.iter().enumerate() {
                if i < inv_slots.len() {
                    inv_slots[i] = Some(item);
                }
            }
            let selected_index = player.selected_inventory_index;
            for i in 0..inv_slots.len() {
                let x = 10;
                let y = 110 + (i as i32 * 25);
                let selected_rect;
                if let Some(entry) = inv_slots[i] {
                    if entry.amount > 1 {
                        let item_text =
                            format!("{} x{}", entity::item_info(entry.item).name, entry.amount);
                        screen.draw_text(&item_text, x as i32, y as i32, 20, Color::WHITE);
                    } else {
                        // If amount is 1, just show the item name
                        let item_text = entity::item_info(entry.item).name;
                        screen.draw_text(item_text, x as i32, y as i32, 20, Color::WHITE);
                    }
                    let item_text = format!("{} {}", i + 1, entity::item_info(entry.item).name);
                    screen.draw_text(&item_text, x as i32, y as i32, 20, Color::WHITE);
                    // Highlight the selected item
                    selected_rect = Rectangle::new(
                        x as f32 - 5.0,
                        y as f32,
                        screen.measure_text(&item_text, 20) as f32 + 10.0,
                        20.0,
                    );
                } else {
                    screen.draw_text("-", x as i32, y as i32, 20, Color::GRAY);
                    selected_rect = Rectangle::new(x as f32 - 5.0, y as f32, 60.0, 20.0);
                }

                // Draw selection rectangle
                if i == selected_index {
                    screen.draw_rectangle_lines_ex(selected_rect, 2.0, Color::WHITE);
                }
            }
        }
    }
}
