use raylib::{
    color::Color,
    math::Vector2,
    prelude::{RaylibDraw, RaylibDrawHandle, RaylibTextureMode},
};

use crate::{render::TILE_SIZE, utils::new_york_dist};

/// Generic helper to draw a filled area based on Manhattan distance.
///
/// # Arguments
/// * `d` - The raylib drawing handle for a texture.
/// * `center_tile` - The tile coordinate to start the calculation from.
/// * `range` - The Manhattan distance (number of steps) to fill.
/// * `color` - The color to fill the tiles with.
pub fn draw_manhattan_range_fill(
    d: &mut RaylibTextureMode<RaylibDrawHandle>,
    center_tile: glam::IVec2,
    range: i32,
    color: Color,
) {
    if range <= 0 {
        return;
    }

    for x_offset in -range..=range {
        for y_offset in -range..=range {
            let current_tile = center_tile + glam::IVec2::new(x_offset, y_offset);

            // Check if the current tile is within the Manhattan distance
            if new_york_dist(center_tile, current_tile) <= range {
                d.draw_rectangle(
                    current_tile.x * TILE_SIZE as i32,
                    current_tile.y * TILE_SIZE as i32,
                    TILE_SIZE as i32,
                    TILE_SIZE as i32,
                    color,
                );
            }
        }
    }
}

/// Generic helper to draw an outline around a Manhattan distance area.
///
/// # Arguments
/// * `d` - The raylib drawing handle for a texture.
/// * `center_tile` - The tile coordinate to start the calculation from.
/// * `range` - The Manhattan distance (number of steps) to outline.
/// * `thickness` - The thickness of the outline border.
/// * `color` - The color of the outline.
pub fn draw_manhattan_range_outline(
    d: &mut RaylibTextureMode<RaylibDrawHandle>,
    center_tile: glam::IVec2,
    range: i32,
    thickness: f32,
    color: Color,
) {
    if range <= 0 {
        return;
    }

    for x_offset in -range..=range {
        for y_offset in -range..=range {
            let current_pos = center_tile + glam::IVec2::new(x_offset, y_offset);

            // First, check if the current tile is itself in range.
            if new_york_dist(center_tile, current_pos) <= range {
                // This tile is valid. Now check its four neighbors to see if we need to draw a border.
                let top_left_glam = current_pos.as_vec2() * TILE_SIZE;
                let top_right_glam = (current_pos + glam::IVec2::new(1, 0)).as_vec2() * TILE_SIZE;
                let bottom_left_glam = (current_pos + glam::IVec2::new(0, 1)).as_vec2() * TILE_SIZE;
                let bottom_right_glam =
                    (current_pos + glam::IVec2::new(1, 1)).as_vec2() * TILE_SIZE;

                let top_left_px = Vector2::new(top_left_glam.x, top_left_glam.y);
                let top_right_px = Vector2::new(top_right_glam.x, top_right_glam.y);
                let bottom_left_px = Vector2::new(bottom_left_glam.x, bottom_left_glam.y);
                let bottom_right_px = Vector2::new(bottom_right_glam.x, bottom_right_glam.y);

                // Check neighbor ABOVE
                if new_york_dist(center_tile, current_pos + glam::IVec2::new(0, -1)) > range {
                    d.draw_line_ex(top_left_px, top_right_px, thickness, color);
                }
                // Check neighbor BELOW
                if new_york_dist(center_tile, current_pos + glam::IVec2::new(0, 1)) > range {
                    d.draw_line_ex(bottom_left_px, bottom_right_px, thickness, color);
                }
                // Check neighbor LEFT
                if new_york_dist(center_tile, current_pos + glam::IVec2::new(-1, 0)) > range {
                    d.draw_line_ex(top_left_px, bottom_left_px, thickness, color);
                }
                // Check neighbor RIGHT
                if new_york_dist(center_tile, current_pos + glam::IVec2::new(1, 0)) > range {
                    d.draw_line_ex(top_right_px, bottom_right_px, thickness, color);
                }
            }
        }
    }
}

/// Generic helper to draw a filled "ring" based on an inclusive min and max Manhattan distance.
pub fn draw_manhattan_ring_fill(
    d: &mut RaylibTextureMode<RaylibDrawHandle>,
    center_tile: glam::IVec2,
    min_range: i32,
    max_range: i32,
    color: Color,
) {
    if max_range < 0 || min_range > max_range {
        return;
    }

    for x_offset in -max_range..=max_range {
        for y_offset in -max_range..=max_range {
            let current_tile = center_tile + glam::IVec2::new(x_offset, y_offset);
            let dist = new_york_dist(center_tile, current_tile);

            // CORRECTED LOGIC: Use >= to make the minimum range inclusive.
            if dist >= min_range && dist <= max_range {
                d.draw_rectangle(
                    current_tile.x * TILE_SIZE as i32,
                    current_tile.y * TILE_SIZE as i32,
                    TILE_SIZE as i32,
                    TILE_SIZE as i32,
                    color,
                );
            }
        }
    }
}

/// Generic helper to draw an outline around a Manhattan distance "ring".
pub fn draw_manhattan_ring_outline(
    d: &mut RaylibTextureMode<RaylibDrawHandle>,
    center_tile: glam::IVec2,
    min_range: i32,
    max_range: i32,
    thickness: f32,
    color: Color,
) {
    if max_range < 0 || min_range > max_range {
        return;
    }

    for x_offset in -max_range..=max_range {
        for y_offset in -max_range..=max_range {
            let current_pos = center_tile + glam::IVec2::new(x_offset, y_offset);
            let dist = new_york_dist(center_tile, current_pos);

            // Check if the current tile is itself in the valid ring.
            if dist >= min_range && dist <= max_range {
                let top_left_px = Vector2::new(
                    current_pos.x as f32 * TILE_SIZE,
                    current_pos.y as f32 * TILE_SIZE,
                );
                let top_right_px = Vector2::new(
                    (current_pos.x + 1) as f32 * TILE_SIZE,
                    current_pos.y as f32 * TILE_SIZE,
                );
                let bottom_left_px = Vector2::new(
                    current_pos.x as f32 * TILE_SIZE,
                    (current_pos.y + 1) as f32 * TILE_SIZE,
                );
                let bottom_right_px = Vector2::new(
                    (current_pos.x + 1) as f32 * TILE_SIZE,
                    (current_pos.y + 1) as f32 * TILE_SIZE,
                );

                // A border is drawn if the neighbor is outside the valid ring.
                // CORRECTED LOGIC: Check against the inclusive min_range and max_range.

                // Check neighbor ABOVE
                let neighbor_above_dist =
                    new_york_dist(center_tile, current_pos + glam::IVec2::new(0, -1));
                if neighbor_above_dist < min_range || neighbor_above_dist > max_range {
                    d.draw_line_ex(top_left_px, top_right_px, thickness, color);
                }
                // Check neighbor BELOW
                let neighbor_below_dist =
                    new_york_dist(center_tile, current_pos + glam::IVec2::new(0, 1));
                if neighbor_below_dist < min_range || neighbor_below_dist > max_range {
                    d.draw_line_ex(bottom_left_px, bottom_right_px, thickness, color);
                }
                // Check neighbor LEFT
                let neighbor_left_dist =
                    new_york_dist(center_tile, current_pos + glam::IVec2::new(-1, 0));
                if neighbor_left_dist < min_range || neighbor_left_dist > max_range {
                    d.draw_line_ex(top_left_px, bottom_left_px, thickness, color);
                }
                // Check neighbor RIGHT
                let neighbor_right_dist =
                    new_york_dist(center_tile, current_pos + glam::IVec2::new(1, 0));
                if neighbor_right_dist < min_range || neighbor_right_dist > max_range {
                    d.draw_line_ex(top_right_px, bottom_right_px, thickness, color);
                }
            }
        }
    }
}
