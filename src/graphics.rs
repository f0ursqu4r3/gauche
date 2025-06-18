/* Asset Loading/Unloading, caching, fetching, and camera state are here.
   responsible for holding all loaded graphical assets.
*/

use crate::sprite::Sprite;
use glam::*;
use raylib::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use strum::IntoEnumIterator;

pub const SPRITE_ASSETS_FOLDER: &str = "./assets/graphics/";

pub struct PlayCam {
    pub pos: Vec2,
}

pub struct Graphics {
    // Window and rendering dimensions
    pub window_dims: glam::UVec2,
    pub dims: glam::UVec2, // This is the internal rendering resolution
    pub fullscreen: bool,

    // Camera
    pub camera: Camera2D,
    pub play_cam: PlayCam,

    // Asset storage
    pub sprite_textures: HashMap<Sprite, Texture2D>,
    pub shaders: Vec<Shader>,
}

impl Graphics {
    pub fn new(rl: &mut RaylibHandle, rlt: &RaylibThread) -> Result<Self, String> {
        let sprite_textures = load_sprite_textures(rl, rlt, SPRITE_ASSETS_FOLDER)?;

        // --- Window and Resolution Setup ---
        // The window_dims is the actual OS window size.
        let window_dims = UVec2::new(1280, 720);
        let fullscreen = false;

        // The internal rendering resolution (`dims`). You can set this to a lower
        // value for a retro pixel-art aesthetic. The result is then scaled up to the window size.
        let dims = UVec2::new(1280, 720);
        // let dims = UVec2::new(1920, 1080); // 1080p
        // let dims = UVec2::new(640, 360);   // nHD
        // let dims = UVec2::new(480, 272);   // Sony PSP
        // let dims = UVec2::new(240, 160);   // Nintendo GBA

        rl.set_window_size(window_dims.x as i32, window_dims.y as i32);
        if fullscreen {
            rl.toggle_fullscreen();
            // In fullscreen, you might want the window dims to match the monitor
        }

        // Center the window on the primary monitor.
        let current_monitor = get_current_monitor();
        let monitor_width = get_monitor_width(current_monitor);
        let monitor_height = get_monitor_height(current_monitor);
        let offset_x = (monitor_width / 2) - (window_dims.x as i32 / 2);
        let offset_y = (monitor_height / 2) - (window_dims.y as i32 / 2);
        rl.set_window_position(offset_x, offset_y);
        rl.set_target_fps(144);

        // --- Shader Loading ---
        let mut shaders = Vec::new();
        let shader_names = vec!["grayscale.fs"]; // Add any other shader files here
        for name in shader_names {
            // This line calls the helper and uses `?` to get the `Shader` out of the `Result`.
            // If loading fails, the `?` will make the whole `Graphics::new` function return the error.
            shaders.push(load_shader(rl, rlt, name)?);
        }

        // --- Camera Setup ---
        let camera = Camera2D {
            target: raylib::math::Vector2::new(0.0, 0.0),
            offset: raylib::math::Vector2::new((dims.x / 2) as f32, (dims.y / 2) as f32),
            rotation: 0.0,
            zoom: 1.0, // Start with 1.0 zoom and adjust based on dims
        };

        Ok(Self {
            window_dims,
            dims,
            fullscreen,
            camera,
            play_cam: PlayCam { pos: Vec2::ZERO },
            sprite_textures,
            shaders,
        })
    }

    /// Safely gets a reference to a loaded sprite texture from the HashMap.
    pub fn get_sprite_texture(&self, sprite: Sprite) -> Option<&Texture2D> {
        self.sprite_textures.get(&sprite)
    }
}

/// Loads all textures defined in the `Sprite` enum into a HashMap.
fn load_sprite_textures(
    rl: &mut RaylibHandle,
    rlt: &RaylibThread,
    asset_folder: &str,
) -> Result<HashMap<Sprite, Texture2D>, String> {
    let mut textures = HashMap::new();
    println!("--- Loading Sprites from: '{}' ---", asset_folder);

    for sprite in Sprite::iter() {
        if sprite == Sprite::NoSprite {
            continue;
        }

        let filename: &'static str = sprite.into();
        let png_path = Path::new(asset_folder).join(format!("{}.png", filename));

        let texture = rl
            .load_texture(rlt, png_path.to_str().unwrap())
            .map_err(|e| {
                format!(
                    "Failed to load texture for {:?} (from {}): {}",
                    sprite, filename, e
                )
            })?;

        println!("- Loaded: {:?}", png_path);
        textures.insert(sprite, texture);
    }
    println!("--- {} sprites loaded successfully. ---", textures.len());
    Ok(textures)
}

/// Helper function to load a single shader from the `src/shaders` directory.
/// Returns a Result containing the Shader or an error String.
fn load_shader(
    rl: &mut RaylibHandle,
    rlt: &RaylibThread,
    filename: &str,
) -> Result<Shader, String> {
    let path = format!("src/shaders/{}", filename);

    // As per the raylib C API, load_shader returns a Shader object directly.
    let shader = rl.load_shader(rlt, None, Some(&path));

    // A shader is considered invalid if its ID is 0. We check this to determine
    // if the load was successful and then construct our own Result.
    if shader.id > 0 {
        println!("- Loaded Shader: {}", path);
        Ok(shader)
    } else {
        Err(format!("Failed to load shader from path: '{}'", path))
    }
}
