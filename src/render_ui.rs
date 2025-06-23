use glam::Vec2;
use raylib::{
    color::Color,
    math::{Rectangle, Vector2},
    prelude::{RaylibDraw, RaylibDrawHandle, RaylibTextureMode},
};

use crate::{
    graphics::Graphics,
    render::TILE_SIZE,
    render_primitives::{draw_manhattan_range_fill, draw_manhattan_range_outline},
    sprite::Sprite,
    state::State,
    utils::new_york_dist,
};

pub fn render_inventory(
    state: &State,
    _graphics: &Graphics,
    screen: &mut RaylibTextureMode<RaylibDrawHandle>,
) {
    if let Some(player_vid) = state.player_vid {
        if let Some(player) = state.entity_manager.get_entity(player_vid) {
            let mut inv_slots = vec![None; 10];
            for item in player.inventory.iter() {
                if item.index < inv_slots.len() {
                    inv_slots[item.index] = Some(item);
                }
            }
            let selected_index = player.inventory.selected_index;
            for i in 0..inv_slots.len() {
                let x = 10;
                let y = 128 + (i as i32 * 25);
                let selected_rect;
                if let Some(entry) = inv_slots[i] {
                    let item_name = entry.item.name;
                    let mut item_text = item_name.to_string();
                    if entry.item.count > 1 {
                        item_text = format!("{} x{}", item_text, entry.item.count);
                    }
                    screen.draw_text(&item_text, x as i32, y as i32, 20, Color::WHITE);
                    selected_rect = Rectangle::new(
                        x as f32 - 6.0,
                        y as f32 - 2.0,
                        screen.measure_text(&item_text, 20) as f32 + 12.0,
                        24.0,
                    );
                } else {
                    screen.draw_text("-", x as i32, y as i32, 20, Color::GRAY);
                    selected_rect = Rectangle::new(x as f32 - 6.0, y as f32 - 2.0, 64.0, 24.0);
                }

                // Draw selection rectangle
                if i == selected_index {
                    screen.draw_rectangle_lines_ex(selected_rect, 2.0, Color::WHITE);
                }
            }
        }
    }
}

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

/// Renders a semi-transparent overlay on all tiles within the player's item range.
/// This is a wrapper around the generic `draw_manhattan_range_fill` function.
pub fn render_item_range_indicator_base(
    d: &mut RaylibTextureMode<RaylibDrawHandle>,
    state: &State,
    _graphics: &Graphics,
) {
    const RANGE_INDICATOR_COLOR: Color = Color::new(40, 40, 40, 40);

    if let Some(player) = state.entity_manager.get_entity(state.player_vid.unwrap()) {
        if let Some(inv_entry) = player.inventory.selected_entry() {
            let range = inv_entry.item.range.round() as i32;
            let player_tile_pos = player.pos.as_ivec2();
            draw_manhattan_range_fill(d, player_tile_pos, range, RANGE_INDICATOR_COLOR);
        }
    }
}

/// Renders a crisp outline around the border of the item's effective range.
/// This is a wrapper around the generic `draw_manhattan_range_outline` function.
pub fn render_item_range_indicator_top(
    d: &mut RaylibTextureMode<RaylibDrawHandle>,
    state: &State,
    _graphics: &Graphics,
) {
    const BORDER_COLOR: Color = Color::new(255, 255, 255, 40);
    const BORDER_THICKNESS: f32 = 1.0;

    if let Some(player) = state.entity_manager.get_entity(state.player_vid.unwrap()) {
        if let Some(inv_entry) = player.inventory.selected_entry() {
            let range = inv_entry.item.range.round() as i32;
            let player_tile_pos = player.pos.as_ivec2();
            draw_manhattan_range_outline(d, player_tile_pos, range, BORDER_THICKNESS, BORDER_COLOR);
        }
    }
}

/// Render hand_item
/// This is a line out from the player to the mouse position
/// for now just a solid line
pub fn render_hand_item(
    d: &mut RaylibTextureMode<RaylibDrawHandle>,
    state: &State,
    graphics: &Graphics,
) {
    if let Some(player_vid) = state.player_vid {
        if let Some(player) = state.entity_manager.get_entity(player_vid) {
            // Get the mouse position in world coordinates
            let mouse_screen_pos = state.mouse_inputs.pos.as_vec2();
            let mouse_world_pos = graphics.screen_to_world(mouse_screen_pos) * TILE_SIZE;
            let player_pos = player.pos * TILE_SIZE;

            // step two.
            /*
                use new york distance.

                draw the item sprite snapped to the nearest tile the mouse is over.
                scaled always to be one quarter tile size at zoom 2.0 just like above
            */
            let mouse_tile_pos = graphics.screen_to_tile(mouse_screen_pos);
            let scale = 0.5; // Scale to 1/4 tile size at zoom 2.0
            let render_size = Vec2::new(TILE_SIZE * scale, TILE_SIZE * scale);
            // Calculate the position to draw the item sprite
            let item_draw_pos = Vec2::new(
                mouse_tile_pos.x as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                mouse_tile_pos.y as f32 * TILE_SIZE + TILE_SIZE / 2.0,
            );
            let new_york_distance = new_york_dist(mouse_tile_pos, player.pos.as_ivec2());
            // Draw the item sprite at the snapped tile position
            if let Some(inv_entry) = player.inventory.selected_entry() {
                if let Some(sprite) = inv_entry.item.sprite {
                    if let Some(texture) = graphics.get_sprite_texture(sprite) {
                        let out_of_range = if inv_entry.item.range > 0.0 {
                            new_york_distance > inv_entry.item.range as i32
                        } else {
                            false // If range is 0, it's considered in range
                        };

                        let tint = if out_of_range {
                            Color::new(50, 50, 50, 200) // Greyed out color
                        } else {
                            Color::WHITE // Normal color
                        };
                        d.draw_texture_pro(
                            texture,
                            Rectangle::new(0.0, 0.0, texture.width as f32, texture.height as f32),
                            Rectangle::new(
                                item_draw_pos.x,
                                item_draw_pos.y,
                                render_size.x,
                                render_size.y,
                            ),
                            Vector2::new(render_size.x / 2.0, render_size.y / 2.0),
                            0.0,
                            tint,
                        );
                    }
                }
            }
        }
    }
}
