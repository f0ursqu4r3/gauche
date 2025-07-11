use glam::Vec2;
use raylib::{
    color::Color,
    math::{Rectangle, Vector2},
    prelude::{RaylibDraw, RaylibDrawHandle, RaylibTextureMode},
};

use crate::{
    entity::EntityType,
    graphics::Graphics,
    item::Item,
    render::TILE_SIZE,
    render_primitives::{
        draw_manhattan_range_fill, draw_manhattan_range_outline, draw_manhattan_ring_fill,
        draw_manhattan_ring_outline,
    },
    sprite::Sprite,
    state::State,
    utils::new_york_dist,
};

pub fn render_inventory(
    state: &State,
    graphics: &Graphics,
    screen: &mut RaylibTextureMode<RaylibDrawHandle>,
) {
    // --- UI Layout & Style Constants ---
    const START_X: f32 = 40.0; // Pushed right to make space for hotkeys
    const START_Y: f32 = 120.0;
    const SLOT_WIDTH: f32 = 200.0;
    const SLOT_HEIGHT: f32 = 30.0;
    const SLOT_SPACING: f32 = 35.0;
    const SELECTION_OFFSET_X: f32 = 25.0;
    const ICON_SIZE: f32 = 24.0;
    const ICON_PADDING: f32 = (SLOT_HEIGHT - ICON_SIZE) / 2.0;
    const FONT_SIZE: i32 = 20;

    const BASE_ANGLE: f32 = -2.0;
    const SELECTED_ANGLE: f32 = 1.0; // Selected item has a different angle

    const BG_COLOR: Color = Color::new(10, 10, 10, 180);
    const ITEM_TEXT_COLOR: Color = Color::WHITE;
    const HOTKEY_COLOR: Color = Color::new(150, 150, 150, 200);

    if let Some(player_vid) = state.player_vid {
        if let Some(player) = state.entity_manager.get_entity(player_vid) {
            let entries: std::collections::HashMap<usize, &crate::inventory::InvEntry> = player
                .inventory
                .entries
                .iter()
                .map(|e| (e.index, e))
                .collect();

            // Always loop up to MAX_SLOTS to draw all 10 slots
            for i in 0..crate::inventory::MAX_SLOTS {
                let is_selected = i == player.inventory.selected_index;
                let y_pos = START_Y + (i as f32 * SLOT_SPACING);

                // --- 1. Draw Hotkey Number ---
                // Map index 9 to "0" for the 10th slot, otherwise it's index + 1
                let hotkey_text = if i == 9 {
                    "0".to_string()
                } else {
                    (i + 1).to_string()
                };
                screen.draw_text(
                    &hotkey_text,
                    (START_X - 20.0) as i32,
                    (y_pos - 10.0) as i32,
                    FONT_SIZE,
                    HOTKEY_COLOR,
                );

                // --- 2. Calculate position and angle ---
                let (x_pos, angle) = if is_selected {
                    (START_X + SELECTION_OFFSET_X, SELECTED_ANGLE)
                } else {
                    (START_X, BASE_ANGLE)
                };

                // --- 3. Draw Angled Background ---
                let bg_rect = Rectangle::new(x_pos, y_pos, SLOT_WIDTH, SLOT_HEIGHT);
                let origin = Vector2::new(0.0, SLOT_HEIGHT / 2.0); // Rotate from left-center
                screen.draw_rectangle_pro(bg_rect, origin, angle, BG_COLOR);

                // --- 4. Draw Contents (Icon and Text) ---
                if let Some(entry) = entries.get(&i) {
                    let item = &entry.item;

                    let mut text_start_x = x_pos + ICON_PADDING;

                    // Draw Icon (if it exists)
                    if let Some(sprite) = item.sprite {
                        if let Some(texture) = graphics.get_sprite_texture(sprite) {
                            let icon_pos_x = x_pos + ICON_PADDING;
                            let icon_pos_y = y_pos - (ICON_SIZE / 2.0);
                            screen.draw_texture(
                                texture,
                                icon_pos_x as i32,
                                icon_pos_y as i32,
                                Color::WHITE,
                            );

                            text_start_x = icon_pos_x + ICON_SIZE + ICON_PADDING;
                        }
                    }

                    // Draw Text
                    let count_text = if item.count > 1 {
                        format!("x{}", item.count)
                    } else {
                        "".to_string()
                    };
                    let full_text = format!("{} {}", item.name, count_text);
                    let text_y_pos = y_pos - (FONT_SIZE as f32 / 2.0);
                    screen.draw_text(
                        &full_text,
                        text_start_x as i32,
                        text_y_pos as i32,
                        FONT_SIZE,
                        ITEM_TEXT_COLOR,
                    );
                }
                // If the slot is empty, we simply don't draw anything inside it.
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
pub fn render_item_range_indicator_base(
    d: &mut RaylibTextureMode<RaylibDrawHandle>,
    state: &State,
    _graphics: &Graphics,
) {
    const RANGE_INDICATOR_COLOR: Color = Color::new(40, 40, 40, 40);

    if let Some(player_vid) = state.player_vid {
        if let Some(player) = state.entity_manager.get_entity(player_vid) {
            if let Some(inv_entry) = player.inventory.selected_entry() {
                let min_range = inv_entry.item.min_range.round() as i32;
                let max_range = inv_entry.item.range.round() as i32;
                let player_tile_pos = player.pos.as_ivec2();

                draw_manhattan_ring_fill(
                    d,
                    player_tile_pos,
                    min_range,
                    max_range,
                    RANGE_INDICATOR_COLOR,
                );
            }
        }
    }
}

/// Renders a crisp outline around the border of the item's effective range.
pub fn render_item_range_indicator_top(
    d: &mut RaylibTextureMode<RaylibDrawHandle>,
    state: &State,
    _graphics: &Graphics,
) {
    const BORDER_COLOR: Color = Color::new(255, 255, 255, 40);
    const BORDER_THICKNESS: f32 = 1.0;
    if let Some(player_vid) = state.player_vid {
        if let Some(player) = state.entity_manager.get_entity(player_vid) {
            if let Some(inv_entry) = player.inventory.selected_entry() {
                let min_range = inv_entry.item.min_range.round() as i32;
                let max_range = inv_entry.item.range.round() as i32;
                let player_tile_pos = player.pos.as_ivec2();

                draw_manhattan_ring_outline(
                    d,
                    player_tile_pos,
                    min_range,
                    max_range,
                    BORDER_THICKNESS,
                    BORDER_COLOR,
                );
            }
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

pub fn render_debug_info(
    state: &State,
    graphics: &Graphics,
    screen: &mut RaylibTextureMode<RaylibDrawHandle>,
) {
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
}

// This helper function handles word-wrapping for the description text.
fn draw_text_wrapped(
    d: &mut RaylibTextureMode<RaylibDrawHandle>,
    text: &str,
    mut x: f32,
    mut y: f32,
    max_width: f32,
    font_size: i32,
    line_spacing: f32,
    color: Color,
) {
    let mut current_line = String::new();
    for word in text.split_whitespace() {
        let test_line = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };

        if d.measure_text(&test_line, font_size) as f32 > max_width {
            d.draw_text(&current_line, x as i32, y as i32, font_size, color);
            y += font_size as f32 + line_spacing;
            current_line = word.to_string();
        } else {
            current_line = test_line;
        }
    }
    d.draw_text(&current_line, x as i32, y as i32, font_size, color);
}

/// Renders a details panel for the currently selected item on the right side of the screen.
/// This is a wrapper around the more generic `render_item_details_panel`.
pub fn render_selected_item_details(
    state: &State,
    graphics: &Graphics,
    screen: &mut RaylibTextureMode<RaylibDrawHandle>,
) {
    if state.player_vid.is_none() {
        return;
    }

    // 1. Fetch the item from the player's inventory
    let selected_item =
        if let Some(player) = state.entity_manager.get_entity(state.player_vid.unwrap()) {
            player.inventory.selected_entry().map(|entry| &entry.item) // Get a reference
        } else {
            None
        };

    // 2. If an item is selected, calculate the position and call the generic function
    if let Some(item) = selected_item {
        const PANEL_WIDTH: f32 = 250.0;
        let x_pos = graphics.dims.x as f32 - PANEL_WIDTH - 30.0;
        let y_pos = graphics.dims.y as f32 * 0.6;

        render_item_details_panel(screen, graphics, item, x_pos, y_pos, "Selected");
    }
}

/// Renders a details panel for the item currently below the player. if there is one
/// render_item_below_player(state, graphics, screen);
pub fn render_item_below_player(
    state: &State,
    graphics: &Graphics,
    screen: &mut RaylibTextureMode<RaylibDrawHandle>,
) {
    if state.player_vid.is_none() {
        return;
    }

    if let Some(player_vid) = state.player_vid {
        if let Some(player) = state.entity_manager.get_entity(player_vid) {
            // Get the tile position below the player
            let player_tile_pos = player.pos.as_ivec2();

            // pub spatial_grid: Vec<Vec<Vec<VID>>>,

            // loop through the grid and get any entity which is type_ Item
            let item_below = state
                .spatial_grid
                .get(player_tile_pos.x as usize)
                .and_then(|col| col.get(player_tile_pos.y as usize))
                .and_then(|cell| {
                    cell.iter().find_map(|vid| {
                        state.entity_manager.get_entity(*vid).and_then(|entity| {
                            if entity.type_ == EntityType::Item {
                                Some(entity)
                            } else {
                                None
                            }
                        })
                    })
                });

            // If there's an item below, render its details panel
            if let Some(item_entity) = item_below {
                if let Some(item) = item_entity.item {
                    const PANEL_WIDTH: f32 = 250.0;
                    let x_pos = graphics.dims.x as f32 - PANEL_WIDTH * 2.2 - 30.0;
                    let y_pos = graphics.dims.y as f32 * 0.6;

                    render_item_details_panel(screen, graphics, &item, x_pos, y_pos, "Item Below");
                }
            }
        }
    }
}

/// Generic function to render a compact, themed details panel for any given item at a specific position.
pub fn render_item_details_panel(
    screen: &mut RaylibTextureMode<RaylibDrawHandle>,
    graphics: &Graphics,
    item: &Item,
    x_pos: f32,
    y_pos: f32,
    label: &str,
) {
    // --- UI Layout & Style ---
    const PANEL_WIDTH: f32 = 240.0;
    const PANEL_HEIGHT: f32 = 270.0;
    const PANEL_PADDING: f32 = 12.0;
    const LINE_SPACING: f32 = 4.0;
    const SECTION_SPACING: f32 = 8.0;

    // --- Themed Style & Offsets ---
    const BANNER_OFFSET: Vector2 = Vector2::new(-20.0, -15.0); // <-- Adjust banner position
    const TOP_PANEL_OFFSET: Vector2 = Vector2::new(-8.0, -8.0); // <-- Adjust top panel offset

    const BASE_ANGLE: f32 = 1.0;
    const TOP_ANGLE: f32 = 2.0;
    const BASE_BG_COLOR: Color = Color::new(10, 10, 10, 210);
    const TOP_BG_COLOR: Color = Color::new(25, 25, 25, 220);

    // --- Banner Style ---
    const BANNER_HEIGHT: f32 = 35.0;
    const BANNER_ANGLE: f32 = -2.0;
    const BANNER_COLOR: Color = Color::new(140, 40, 40, 230);
    const BANNER_LABEL_FONT_SIZE: i32 = 20;
    const BANNER_LABEL_COLOR: Color = Color::WHITE;
    const BANNER_SHADOW_COLOR: Color = Color::new(0, 0, 0, 150);

    // --- Font & Text Style ---
    const TITLE_FONT_SIZE: i32 = 22;
    const DESC_FONT_SIZE: i32 = 16;
    const STAT_FONT_SIZE: i32 = 16;
    const TITLE_COLOR: Color = Color::WHITE;
    const DESC_COLOR: Color = Color::new(180, 180, 180, 255);
    const STAT_KEY_COLOR: Color = Color::new(150, 150, 150, 255);
    const STAT_VALUE_COLOR: Color = Color::WHITE;
    const STATUS_READY_COLOR: Color = Color::new(120, 220, 120, 255);
    const STATUS_COOLDOWN_COLOR: Color = Color::new(220, 180, 120, 255);

    // --- 1. Draw Themed Background Layers for the Panel ---
    let panel_y_pos = y_pos + 20.0; // Shift panel down to make room for banner
    let base_rect = Rectangle::new(x_pos, panel_y_pos, PANEL_WIDTH, PANEL_HEIGHT);
    screen.draw_rectangle_pro(base_rect, Vector2::zero(), BASE_ANGLE, BASE_BG_COLOR);

    let top_rect = Rectangle::new(
        x_pos + TOP_PANEL_OFFSET.x,
        panel_y_pos + TOP_PANEL_OFFSET.y,
        PANEL_WIDTH,
        PANEL_HEIGHT,
    );
    screen.draw_rectangle_pro(top_rect, Vector2::zero(), TOP_ANGLE, TOP_BG_COLOR);

    // --- 2. Draw the Banner ---
    let banner_rect = Rectangle::new(
        x_pos + BANNER_OFFSET.x,
        y_pos + BANNER_OFFSET.y,
        PANEL_WIDTH + 20.0,
        BANNER_HEIGHT,
    );
    screen.draw_rectangle_pro(banner_rect, Vector2::zero(), BANNER_ANGLE, BANNER_COLOR);

    // --- 3. Draw Banner Label with Shadow ---
    let text_width = screen.measure_text(label, BANNER_LABEL_FONT_SIZE);
    let text_x = (banner_rect.x + (banner_rect.width / 2.0) - (text_width as f32 / 2.0)) as i32;
    let text_y = (banner_rect.y - 3.0 + (banner_rect.height / 2.0)
        - (BANNER_LABEL_FONT_SIZE as f32 / 2.0)) as i32;

    // Shadow (drawn first)
    screen.draw_text(
        label,
        text_x + 2,
        text_y + 2,
        BANNER_LABEL_FONT_SIZE,
        BANNER_SHADOW_COLOR,
    );
    // Main Text
    screen.draw_text(
        label,
        text_x,
        text_y,
        BANNER_LABEL_FONT_SIZE,
        BANNER_LABEL_COLOR,
    );

    // --- 4. Define Content Area (relative to the top panel layer) ---
    let content_x = top_rect.x + PANEL_PADDING;
    let mut current_y = top_rect.y + PANEL_PADDING;
    let content_width = PANEL_WIDTH - (PANEL_PADDING * 2.0);

    // --- 5. Draw Panel Content ---

    // Item Name & Count
    let title_text = if item.max_count > 1 {
        format!("{} ({} / {})", item.name, item.count, item.max_count)
    } else {
        item.name.to_string()
    };
    screen.draw_text(
        &title_text,
        content_x as i32,
        current_y as i32,
        TITLE_FONT_SIZE,
        TITLE_COLOR,
    );
    current_y += TITLE_FONT_SIZE as f32 + SECTION_SPACING;

    // Item Description
    let desc_height = draw_text_wrapped_and_get_height(
        screen,
        item.description,
        content_x,
        current_y,
        content_width,
        DESC_FONT_SIZE,
        LINE_SPACING,
        DESC_COLOR,
    );
    current_y += desc_height + SECTION_SPACING * 2.0;

    // Core Stats (only shown if relevant)
    current_y = draw_stat_if(
        screen,
        item.range > 0.0,
        "Range",
        &format!("{} - {}", item.min_range.round(), item.range.round()),
        content_x,
        current_y,
    );
    current_y = draw_stat_if(
        screen,
        item.use_cooldown > 0.0,
        "Cooldown",
        &format!("{:.1}s", item.use_cooldown),
        content_x,
        current_y,
    );

    // Status (Live Cooldown)
    if item.use_cooldown > 0.0 {
        let (status_text, status_color) = if item.use_cooldown_countdown > 0.0 {
            (
                format!("{:.1}s", item.use_cooldown_countdown),
                STATUS_COOLDOWN_COLOR,
            )
        } else {
            ("Ready".to_string(), STATUS_READY_COLOR)
        };
        current_y = draw_stat(
            screen,
            "Status",
            &status_text,
            content_x,
            current_y,
            status_color,
        );
    }

    current_y += SECTION_SPACING;

    // Properties
    current_y = draw_stat(
        screen,
        "Consumable",
        if item.consume_on_use { "Yes" } else { "No" },
        content_x,
        current_y,
        STAT_VALUE_COLOR,
    );
    draw_stat(
        screen,
        "Droppable",
        if item.droppable { "Yes" } else { "No" },
        content_x,
        current_y,
        STAT_VALUE_COLOR,
    );
}

// --- HELPER FUNCTIONS ---

/// An enhanced version of draw_text_wrapped that returns the total height of the text block.
fn draw_text_wrapped_and_get_height(
    d: &mut RaylibTextureMode<RaylibDrawHandle>,
    text: &str,
    x: f32,
    y: f32,
    max_width: f32,
    font_size: i32,
    line_spacing: f32,
    color: Color,
) -> f32 {
    let mut y_offset = 0.0;
    let mut current_line = String::new();
    let mut lines = Vec::new();

    if text.is_empty() {
        return 0.0;
    }

    for word in text.split_whitespace() {
        let test_line = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };

        if d.measure_text(&test_line, font_size) as f32 > max_width {
            lines.push(current_line);
            current_line = word.to_string();
        } else {
            current_line = test_line;
        }
    }
    lines.push(current_line);

    for line in lines {
        d.draw_text(&line, x as i32, (y + y_offset) as i32, font_size, color);
        y_offset += font_size as f32 + line_spacing;
    }

    y_offset - line_spacing // Return total height without the last line's spacing
}

/// Helper to draw a key-value stat line and return the new Y position.
fn draw_stat(
    screen: &mut RaylibTextureMode<RaylibDrawHandle>,
    key: &str,
    value: &str,
    x: f32,
    y: f32,
    value_color: Color,
) -> f32 {
    const STAT_FONT_SIZE: i32 = 16;
    const STAT_KEY_COLOR: Color = Color::new(150, 150, 150, 255);
    const LINE_SPACING: f32 = 4.0;

    let key_text = format!("{}: ", key);
    screen.draw_text(
        &key_text,
        x as i32,
        y as i32,
        STAT_FONT_SIZE,
        STAT_KEY_COLOR,
    );
    let key_width = screen.measure_text(&key_text, STAT_FONT_SIZE) as f32;
    screen.draw_text(
        value,
        (x + key_width) as i32,
        y as i32,
        STAT_FONT_SIZE,
        value_color,
    );

    y + STAT_FONT_SIZE as f32 + LINE_SPACING
}

/// Wrapper for draw_stat that only draws if the condition is true.
fn draw_stat_if(
    screen: &mut RaylibTextureMode<RaylibDrawHandle>,
    condition: bool,
    key: &str,
    value: &str,
    x: f32,
    y: f32,
) -> f32 {
    if condition {
        draw_stat(screen, key, value, x, y, Color::WHITE)
    } else {
        y
    }
}
