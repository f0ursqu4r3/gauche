/* This is the render mode dispatcher.
   Based on the state.mode, it will call the appropriate render function.
   Fiddly render logic should probably be elsewhere, since i expect a few different modes.
*/

use glam::{IVec2, Vec2};
use raylib::prelude::*;

use crate::{
    graphics::Graphics,
    state::{Mode, State},
};

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

pub fn render(
    rl: &mut RaylibHandle,
    rlt: &mut RaylibThread,
    state: &mut State,
    graphics: &mut Graphics,
    render_texture: &mut RenderTexture2D,
) {
    let mut draw_handle = rl.begin_drawing(rlt);
    {
        let screen = &mut draw_handle.begin_texture_mode(rlt, render_texture);
        screen.clear_background(Color::DARKBROWN);

        match state.mode {
            Mode::Title => render_title(state, graphics, screen),
            Mode::Settings => render_settings_menu(state, graphics, screen),
            Mode::VideoSettings => render_video_settings_menu(state, graphics, screen),
            Mode::Playing => render_playing(state, graphics, screen),
            Mode::StageTransition => render_stage_transition(state, graphics, screen),
            Mode::GameOver => render_game_over(state, graphics, screen),
            Mode::Win => render_win(state, graphics, screen),
            // _ => {}
        }
    }
    scale_and_blit_render_texture_to_window(&mut draw_handle, graphics, render_texture);
}

pub fn render_title(
    state: &mut State,
    graphics: &mut Graphics,
    screen: &mut RaylibTextureMode<RaylibDrawHandle>,
) {
    let screen_center = (graphics.dims / 2).as_vec2();
    graphics.camera.zoom = 1.0;
    graphics.camera.rotation = 0.0;
    graphics.camera.target = raylib::math::Vector2::new(screen_center.x, screen_center.y);
    graphics.camera.offset = raylib::math::Vector2::new(screen_center.x, screen_center.y);
    {
        let mut d = screen.begin_mode2D(graphics.camera);
        {
            let color = raylib::color::Color::new(38, 43, 68, 255);
            d.clear_background(color);

            let screen_center = (graphics.dims / 2).as_vec2();
            graphics.camera.target = raylib::math::Vector2::new(screen_center.x, screen_center.y);
            // graphics.camera.target = raylib::math::Vector2::new(0.0, 0.0);

            /* we have 3 layers and we are doing basic paralax here
            layer 3 is drawn first, and is the furthest back
            layer 2 is drawn second, and is in the middle
            layer 1 is drawn last, and is in the front, it moves the fastest

            */

            let layer_3 = &graphics.textures[Textures::TitleLayer3 as usize];
            let layer_2 = &graphics.textures[Textures::TitleLayer2 as usize];
            let layer_1 = &graphics.textures[Textures::TitleLayer1 as usize];

            // use time to get paralax offset
            let t = d.get_time() as f32;
            let speed = 1.0;
            let paralax_param = (t * speed).sin(); // range -1 to 1.0

            // layer 3
            let layer_3_max_displacement = 0.0001;
            let layer_3_expansion_frac = 1.2 + 4.0 * layer_3_max_displacement;
            let layer_3_target_size = Vec2::new(
                graphics.dims.x as f32 * layer_3_expansion_frac,
                graphics.dims.y as f32,
            );

            let mut layer_3_pos = -(layer_3_target_size - graphics.dims.as_vec2()) / 2.0;
            layer_3_pos.x += paralax_param * graphics.dims.x as f32 * layer_3_max_displacement;

            // layer 2
            let layer_2_max_displacement = 0.02;
            let layer_2_expansion_frac = 1.0 + 4.0 * layer_2_max_displacement;
            let layer_2_target_size = Vec2::new(
                graphics.dims.x as f32 * layer_2_expansion_frac,
                graphics.dims.y as f32,
            );

            let mut layer_2_pos = -(layer_2_target_size - graphics.dims.as_vec2()) / 2.0;
            layer_2_pos.x += paralax_param * graphics.dims.x as f32 * layer_2_max_displacement;

            // layer 1
            let layer_1_max_displacement = 0.06;
            let layer_1_expansion_frac = 1.0 + 4.0 * layer_1_max_displacement;
            let layer_1_target_size = Vec2::new(
                graphics.dims.x as f32 * layer_1_expansion_frac,
                graphics.dims.y as f32,
            );

            let mut layer_1_pos = -(layer_1_target_size - graphics.dims.as_vec2()) / 2.0;
            layer_1_pos.x += paralax_param * graphics.dims.x as f32 * layer_1_max_displacement;

            // draw layers
            //  //  layer 3
            d.draw_texture_pro(
                &layer_3,
                Rectangle::new(0.0, 0.0, layer_3.width() as f32, layer_3.height() as f32),
                Rectangle::new(
                    layer_3_pos.x,
                    0.0,
                    layer_3_target_size.x,
                    layer_3_target_size.y,
                ),
                Vector2::new(0.0, 0.0),
                0.0,
                Color::WHITE,
            );
            //  //  layer 2
            d.draw_texture_pro(
                &layer_2,
                Rectangle::new(0.0, 0.0, layer_2.width() as f32, layer_2.height() as f32),
                Rectangle::new(
                    layer_2_pos.x,
                    0.0,
                    layer_2_target_size.x,
                    layer_2_target_size.y,
                ),
                Vector2::new(0.0, 0.0),
                0.0,
                Color::WHITE,
            );
            //  //  layer 1
            d.draw_texture_pro(
                &layer_1,
                Rectangle::new(0.0, 0.0, layer_1.width() as f32, layer_1.height() as f32),
                Rectangle::new(
                    layer_1_pos.x,
                    0.0,
                    layer_1_target_size.x,
                    layer_1_target_size.y,
                ),
                Vector2::new(0.0, 0.0),
                0.0,
                Color::WHITE,
            );

            draw_menu_title(&mut d, graphics, "Splonks");

            // this is where we will show the options
            // the currently selected option will be red, the others white
            let ten_percent = (graphics.dims.y as f32 * 0.10) as i32;
            let mut cursor = Vec2::new(graphics.dims.x as f32 * 0.15, graphics.dims.y as f32 * 0.6);

            // draw title options
            for option in TitleMenuOption::iter() {
                let color = if option == state.title_menu_selection {
                    Color::RED
                } else {
                    Color::WHITE
                };

                let option_pos = Vec2::new(cursor.x, cursor.y);
                let text = get_title_menu_option_name(&option);
                let font_size =
                    get_reasonable_font_scale(graphics.dims, crate::graphics::TextType::MenuItem);
                d.draw_text(
                    text,
                    option_pos.x as i32,
                    option_pos.y as i32,
                    font_size,
                    color,
                );
                cursor.y += ten_percent as f32;
            }
        }
    }
}

// let mut d = rl.begin_drawing(&rlt);

// // Draw the game world
// d.clear_background(Color::new(30, 20, 30, 255));
// for row in &game.world.tiles {
//     for tile in row {
//         let color = match tile.tile_type {
//             TileType::Grass => Color::GREEN,
//             TileType::Water => Color::BLUE,
//             TileType::Mountain => Color::GRAY,
//             TileType::Wall => Color::BLACK,
//         };
//         d.draw_rectangle(
//             tile.pos.x as i32 * config.game.tile_size,
//             tile.pos.y as i32 * config.game.tile_size,
//             config.game.tile_size,
//             config.game.tile_size,
//             color,
//         );
//     }
// }

// // Draw player
// d.draw_rectangle(
//     game.player.pos.x as i32 * config.game.tile_size,
//     game.player.pos.y as i32 * config.game.tile_size,
//     config.game.tile_size,
//     config.game.tile_size,
//     Color::WHITE,
// );

// // Draw entities
// for entity in &game.entities {
//     let color = match entity.entity_type {
//         EntityType::Monster => Color::RED,
//         EntityType::Item => Color::YELLOW,
//     };
//     d.draw_rectangle(
//         entity.pos.x as i32 * config.game.tile_size,
//         entity.pos.y as i32 * config.game.tile_size,
//         config.game.tile_size,
//         config.game.tile_size,
//         color,
//     );
// }

// d.draw_text(
//     &format!("Player Health: {}", game.player.health),
//     10,
//     10,
//     20,
//     Color::WHITE,
// );
// d.draw_text(
//     &format!("Player Inventory: {}", game.player.inventory.len()),
//     10,
//     40,
//     20,
//     Color::WHITE,
// );
