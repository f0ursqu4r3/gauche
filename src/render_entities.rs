use glam::Vec2;
use rand::random_range;
use raylib::{
    color::Color,
    math::{Rectangle, Vector2},
    prelude::{RaylibDraw, RaylibDrawHandle, RaylibMode2D, RaylibTextureMode},
};

use crate::{
    entity::EntityType,
    graphics::Graphics,
    render::{get_alpha_from_distance, TILE_SIZE, VIEW_DISTANCE},
    state::State,
    tile::get_tile_sprite,
};

/// Renders a health bar above a single non-player entity if it's damaged.
pub fn render_entity_health_bar(
    d: &mut RaylibTextureMode<RaylibDrawHandle>,
    entity: &crate::entity::Entity,
    alpha: u8,
) {
    // Don't draw for the player, or if the entity is at full health or has no health.
    if entity.type_ == EntityType::Player || entity.max_hp == 0 || entity.health == entity.max_hp {
        return;
    }

    let health_percentage = entity.health as f32 / entity.max_hp as f32;

    const BAR_HEIGHT: f32 = 2.0;
    const BAR_WIDTH: f32 = TILE_SIZE * 0.8; // Make it slightly smaller than the tile.
    const Y_OFFSET: f32 = TILE_SIZE * 0.5; // Position it just above the entity's center.

    // Use the entity's alpha so the bar fades with it.
    let bar_bg_color = Color::new(80, 20, 20, (alpha as f32 * 0.8) as u8);
    let bar_fg_color = Color::new(40, 180, 40, (alpha as f32 * 0.9) as u8);

    let entity_pixel_pos = entity.pos * TILE_SIZE;

    // Center the bar over the entity.
    let bar_pos_x = entity_pixel_pos.x - (BAR_WIDTH / 2.0);
    let bar_pos_y = entity_pixel_pos.y - Y_OFFSET;

    // Draw health bar background.
    d.draw_rectangle(
        bar_pos_x as i32,
        bar_pos_y as i32,
        BAR_WIDTH as i32,
        BAR_HEIGHT as i32,
        bar_bg_color,
    );

    // Draw health bar foreground.
    d.draw_rectangle(
        bar_pos_x as i32,
        bar_pos_y as i32,
        (BAR_WIDTH * health_percentage) as i32,
        BAR_HEIGHT as i32,
        bar_fg_color,
    );
}

/// Iterates through all active entities and renders them and their health bars.
pub fn render_entities(
    d: &mut RaylibTextureMode<RaylibDrawHandle>,
    state: &State,
    graphics: &Graphics,
    player_pos_pixels: Option<Vec2>,
) {
    for entity in state.entity_manager.iter().filter(|e| e.active) {
        // if entity has no sprite, skip rendering
        if entity.sprite.is_none() {
            continue;
        }

        // Player is always fully visible; other entities fade with distance.
        let alpha = if entity.type_ == EntityType::Player {
            255
        } else if let Some(player_pos) = player_pos_pixels {
            get_alpha_from_distance(player_pos, entity.pos * TILE_SIZE, VIEW_DISTANCE)
        } else {
            255 // If no player, everything is fully visible.
        };

        // Only draw the entity if it's visible.
        if alpha > 0 {
            let sprite = entity.sprite.unwrap();
            if let Some(texture) = graphics.get_sprite_texture(sprite) {
                let entity_pixel_pos = entity.pos * TILE_SIZE;
                let source_rec =
                    Rectangle::new(0.0, 0.0, texture.width as f32, texture.height as f32);

                // Apply screen shake offset.
                let position = if entity.shake > 0.0 {
                    let shake_offset = entity.shake * TILE_SIZE;
                    let shake_x = random_range(-shake_offset..shake_offset);
                    let shake_y = random_range(-shake_offset..shake_offset);
                    entity_pixel_pos + Vec2::new(shake_x, shake_y)
                } else {
                    entity_pixel_pos
                };

                // Calculate the final render size in pixels using the entity's size property.
                let render_size_pixels = entity.size * TILE_SIZE;

                // The destination rectangle now uses the calculated render size.
                let dest_rec = Rectangle::new(
                    position.x,
                    position.y,
                    render_size_pixels.x,
                    render_size_pixels.y,
                );

                // The origin for rotation is the center of the final rendered size.
                let origin = Vector2::new(render_size_pixels.x / 2.0, render_size_pixels.y / 2.0);

                d.draw_texture_pro(
                    texture,
                    source_rec,
                    dest_rec,
                    origin,
                    entity.rot,
                    Color::new(255, 255, 255, alpha),
                );
            }

            render_entity_health_bar(d, entity, alpha);
        }
    }
}
