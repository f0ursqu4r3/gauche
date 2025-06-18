/* Asset Loading/ Unloading, caching, fetching, camera state is here.
   This should mostly be contained state, graphics settings, and not have any logic.
   Large rendering functions go in their own files.
*/

use crate::{
    sprite::{Sprite, SpriteData},
    tile::Tile,
};

use glam::*;
use raylib::prelude::*;
use strum::{EnumCount, IntoEnumIterator};

use std::{collections::HashMap, path::Path};

pub enum Shaders {
    Grayscale,
}

pub struct PlayCam {
    pub pos: Vec2,
    pub vel: Vec2,
    pub acc: Vec2,
}

pub struct Graphics {
    pub window_dims: glam::UVec2,
    pub dims: glam::UVec2,
    pub fullscreen: bool,

    pub camera: Camera2D,
    pub play_cam: PlayCam,

    pub sprites: Vec<SpriteData>,
    pub sprite_textures: Vec<Texture2D>,
    pub textures: Vec<Texture2D>,

    pub shaders: Vec<Shader>,
}

impl Graphics {
    pub fn new(
        rl: &mut RaylibHandle,
        rlt: &RaylibThread,
        sprite_assets_folder: &str,
    ) -> Result<Self, String> {
        let sprites = load_sprites(sprite_assets_folder)?;
        let sprite_textures = load_sprite_textures(rl, rlt, sprite_assets_folder)?;

        let window_dims = UVec2::new(1280, 720);
        let dims = UVec2::new(1280, 720);
        // let window_dims = UVec2::new(1920, 1080);
        let fullscreen = false;

        // let dims = UVec2::new(800, 600);
        // let dims = UVec2::new(480, 272); //psp
        // let dims = UVec2::new(240, 160); // gba
        // let dims = UVec2::new(128, 128);
        // let dims = UVec2::new(1920, 1080);

        let mut shaders = vec![];

        // LOAD SHADERS
        let texture_names = vec!["grayscale.fs"];
        for name in texture_names {
            let path = format!("src/shaders/{}", name);
            match rl.load_shader(&rlt, None, Some(&path)) {
                Ok(shader) => shaders.push(shader),
                Err(e) => {
                    println!("Error loading shader: {}", e);
                    std::process::exit(1);
                }
            };
        }

        rl.set_window_size(window_dims.x as i32, window_dims.y as i32);
        // fullscreen
        if fullscreen {
            rl.toggle_fullscreen();
            rl.set_window_size(rl.get_screen_width(), rl.get_screen_height());
        }

        // after resetting the window size, we should recenter it or its gonna be in a weird gameplace on the screen
        let current_monitor = get_current_monitor();
        let monitor_width = get_monitor_width(current_monitor);
        let monitor_height = get_monitor_height(current_monitor);
        let screen_dims = UVec2::new(monitor_width as u32, monitor_height as u32);
        let screen_center = screen_dims / 2;
        let window_center = window_dims / 2;

        // print screen dims, screen center, window center
        // let offset = screen_center - window_center;
        // on linux the virtual screen resolution causes this to be wrong, so for development we can set a manual offset
        let offset = UVec2::new(screen_center.x - window_center.x, 600);
        rl.set_window_position(offset.x as i32, offset.y as i32);
        rl.set_target_fps(144);

        let mouse_scale = dims / window_dims;
        rl.set_mouse_scale(mouse_scale.x as f32, mouse_scale.y as f32);

        // let frame_buffer = match rl.load_render_texture(rlt, dims.x, dims.y) {
        //     Ok(rt) => rt,
        //     Err(e) => {
        //         println!("Error loading render texture: {}", e);
        //         std::process::exit(1);
        //     }
        // };

        // LOAD LARGE TEXTURES
        let texture_error = "Error loading texture";
        let mut textures = Vec::new();
        let texture_names = vec!["title", "title_layer_1", "title_layer_2", "title_layer_3"];
        for name in texture_names {
            let path = format!("assets/graphics/images/{}.png", name);
            let texture = rl.load_texture(rlt, &path).expect(texture_error);
            textures.push(texture);
        }

        // LOAD TILE TEXTURES FOR STAGES
        let texture_error = "Error loading tile set texture";
        let mut tile_sets = Vec::new();
        let tile_set_names = vec!["cave", "ice", "jungle", "temple", "boss"];
        for name in tile_set_names {
            let path = format!("assets/graphics/tiles/{}.png", name);
            let texture = rl.load_texture(rlt, &path).expect(texture_error);
            tile_sets.push(texture);
        }

        // LOAD SPECIAL_EFFECTS_ TEXTURE
        let texture_error = "Error loading special effects texture";
        let path = "assets/graphics/special_effects/special_effects.png";
        let special_effects_texture = rl.load_texture(rlt, &path).expect(texture_error);

        let screen_center = (window_dims / 2).as_vec2();
        let camera = Camera2D {
            target: raylib::math::Vector2::new(0.0, 0.0), // doesnt matter because were gonna move this every frame
            offset: raylib::math::Vector2::new(screen_center.x, screen_center.y), // makes what camera targets in the middle of the screen
            rotation: 0.0,
            zoom: 3.0,
        };

        Ok(Self {
            window_dims,
            dims,
            fullscreen, //TODO: make this load from a file lol
            camera,
            play_cam: PlayCam {
                pos: Vec2::ZERO,
                vel: Vec2::ZERO,
                acc: Vec2::ZERO,
            },
            sprites,
            sprite_textures,
            textures,
            shaders,
        })
    }

    pub fn reload_sprite_data_and_textures(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        asset_folder: &str,
    ) -> Result<(), String> {
        self.sprites = load_sprites(asset_folder)?;
        // The old textures will be automatically unloaded when replaced
        self.textures = load_sprite_textures(rl, thread, asset_folder)?;
        Ok(())
    }

    pub fn get_sprite_texture(&self, sprite: Sprite) -> &Texture2D {
        &self.sprite_textures[sprite as usize]
    }

    pub fn get_sprite_data(&self, sprite: Sprite) -> &SpriteData {
        &self.sprites[sprite as usize]
    }

    pub fn screen_to_wc(&self, screen_pos: UVec2) -> Vec2 {
        let screen_pos = screen_pos.as_vec2();
        let screen_center = self.window_dims.as_vec2() / 2.0;
        let screen_pos = screen_pos - screen_center;
        let screen_pos = screen_pos / self.camera.zoom;
        let screen_pos = screen_pos + Vec2::new(self.camera.target.x, self.camera.target.y);
        screen_pos
    }

    pub fn screen_to_tile_coords(&self, screen_pos: UVec2) -> IVec2 {
        let wc = self.screen_to_wc(screen_pos);
        let tile_pos = wc.as_ivec2() / Tile::SIZE as i32;
        tile_pos
    }
}

fn load_sprite_textures(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    asset_folder: &str,
) -> Result<Vec<Texture2D>, String> {
    let mut textures = Vec::with_capacity(Sprite::COUNT);
    for sprite in Sprite::iter() {
        let filename = sprite.to_filename();
        let png_path = Path::new(asset_folder).join(format!("{}.png", filename));
        let texture = rl
            .load_texture(thread, png_path.to_str().unwrap())
            .map_err(|e| format!("Failed to load texture {}: {}", filename, e))?;
        textures.push(texture);
    }
    Ok(textures)
}

pub enum TextType {
    MenuTitle,
    MenuItem,
}

pub fn get_reasonable_font_scale(dims: UVec2, test_type: TextType) -> i32 {
    match dims {
        UVec2 { x: 160, y: 144 } => match test_type {
            TextType::MenuTitle => 60 * dims.y as i32 / 720,
            TextType::MenuItem => 40 * dims.y as i32 / 720,
        },
        _ => match test_type {
            TextType::MenuTitle => 100 * dims.y as i32 / 720,
            TextType::MenuItem => 60 * dims.y as i32 / 720,
        },
    }
}
