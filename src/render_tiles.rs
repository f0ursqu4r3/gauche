use glam::Vec2;
use raylib::{
    color::Color,
    math::{Rectangle, Vector2},
    prelude::{RaylibDraw, RaylibDrawHandle, RaylibMode2D, RaylibTextureMode},
};

use crate::{
    graphics::Graphics,
    render::{TILE_SIZE, VIEW_DISTANCE},
    state::State,
    tile::get_tile_sprite,
};

/// Renders the health bar for a single tile if it's damaged.
pub fn render_tile_health_bar(
    d: &mut RaylibTextureMode<RaylibDrawHandle>,
    tile_data: &crate::stage::TileData,
    tile_pixel_pos: Vec2,
    alpha: u8,
) {
    // Only draw if the tile is breakable and has taken damage.
    if !tile_data.breakable || tile_data.hp == tile_data.max_hp || tile_data.max_hp == 0 {
        return;
    }

    let health_percentage = tile_data.hp as f32 / tile_data.max_hp as f32;

    const BAR_HEIGHT: f32 = 2.0;
    let bar_width = TILE_SIZE;

    // Use the tile's alpha so the bar fades with the tile
    let bar_bg_color = Color::new(80, 20, 20, (alpha as f32 * 0.8) as u8);
    let bar_fg_color = Color::new(40, 180, 40, (alpha as f32 * 0.9) as u8);

    // Position the bar at the top of the tile
    let bar_pos_x = tile_pixel_pos.x;
    let bar_pos_y = tile_pixel_pos.y;

    // Draw health bar background
    d.draw_rectangle(
        bar_pos_x as i32,
        bar_pos_y as i32,
        bar_width as i32,
        BAR_HEIGHT as i32,
        bar_bg_color,
    );

    // Draw health bar foreground, scaled by health percentage
    d.draw_rectangle(
        bar_pos_x as i32,
        bar_pos_y as i32,
        (bar_width * health_percentage) as i32,
        BAR_HEIGHT as i32,
        bar_fg_color,
    );
}

/// Iterates through the stage and renders all visible tiles and their health bars.
/// Iterates through the stage and renders all visible tiles and their health bars.
pub fn render_tiles(
    d: &mut RaylibTextureMode<RaylibDrawHandle>,
    state: &State,
    graphics: &Graphics,
    player_pos_pixels: Option<Vec2>,
) {
    for y in 0..state.stage.get_height() {
        'row: for x in 0..state.stage.get_width() {
            if let Some(tile_data) = state.stage.get_tile(x, y) {
                let tile_pixel_pos = Vec2::new(x as f32, y as f32) * TILE_SIZE;

                let sprite = match get_tile_sprite(&tile_data) {
                    Some(s) => s,
                    None => continue 'row, // Skip if tile has no sprite (e.g., Tile::None)
                };

                if let Some(texture) = graphics.get_sprite_texture(sprite) {
                    // Calculate alpha based on distance from player for a fog-of-war effect.
                    let alpha = if let Some(player_pos) = player_pos_pixels {
                        let distance = (tile_pixel_pos - player_pos).length();
                        let tile_distance = (distance / TILE_SIZE).floor() as u32;
                        let max_steps = (VIEW_DISTANCE / TILE_SIZE) as u32;

                        if tile_distance >= max_steps {
                            0
                        } else {
                            // Alpha falls off linearly from 255 to 0 based on distance.
                            (((max_steps - tile_distance) as f32 / max_steps as f32) * 255.0) as u8
                        }
                    } else {
                        255 // If there's no player, everything is fully visible.
                    };

                    // Only draw the tile and its health bar if it's visible at all.
                    if alpha > 0 {
                        let source_rec =
                            Rectangle::new(0.0, 0.0, texture.width as f32, texture.height as f32);

                        // The destination rectangle's x/y should be the *center* of the tile for rotation.
                        let dest_rec = Rectangle::new(
                            tile_pixel_pos.x + (TILE_SIZE / 2.0),
                            tile_pixel_pos.y + (TILE_SIZE / 2.0),
                            TILE_SIZE,
                            TILE_SIZE,
                        );

                        // The origin for rotation is the center of the sprite itself.
                        let origin = Vector2::new(TILE_SIZE / 2.0, TILE_SIZE / 2.0);

                        d.draw_texture_pro(
                            texture,
                            source_rec,
                            dest_rec,
                            origin,
                            tile_data.rot, // Use the rotation from tile_data
                            Color::new(255, 255, 255, alpha),
                        );

                        // Call the dedicated function to render the health bar (it is not rotated).
                        render_tile_health_bar(d, &tile_data, tile_pixel_pos, alpha);
                    }
                }
            }
        }
    }
}
