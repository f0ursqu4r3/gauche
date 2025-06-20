/* Asset Loading/Unloading, caching, fetching, and camera state are here.
   responsible for holding all loaded graphical assets.
*/

use crate::render::TILE_SIZE;
use crate::sprite::Sprite;
use glam::*;
use raylib::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use strum::IntoEnumIterator;

pub const SPRITE_ASSETS_FOLDER: &str = "./assets/graphics/";

pub struct PlayCam {
    pub pos: Vec2,
    pub zoom: f32,
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

        center_window(rl, window_dims.x as i32, window_dims.y as i32);

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
        let initial_zoom = 2.0;
        let camera = Camera2D {
            target: raylib::math::Vector2::new(0.0, 0.0),
            offset: raylib::math::Vector2::new((dims.x / 2) as f32, (dims.y / 2) as f32),
            rotation: 0.0,
            zoom: initial_zoom,
        };

        Ok(Self {
            window_dims,
            dims,
            fullscreen,
            camera,
            play_cam: PlayCam {
                pos: Vec2::ZERO,
                zoom: initial_zoom,
            },
            sprite_textures,
            shaders,
        })
    }

    /// Safely gets a reference to a loaded sprite texture from the HashMap.
    pub fn get_sprite_texture(&self, sprite: Sprite) -> Option<&Texture2D> {
        self.sprite_textures.get(&sprite)
    }

    pub fn screen_wc(&self, screen_pos: Vec2) -> Vec2 {
        // Convert screen coordinates to world coordinates using the camera's zoom and offset.
        let cam_offset = Vec2::new(self.camera.offset.x, self.camera.offset.y);
        let zoomed_pos = (screen_pos - cam_offset) / self.camera.zoom;
        let cam_target = Vec2::new(self.camera.target.x, self.camera.target.y);
        zoomed_pos + cam_target
    }

    pub fn screen_tc(&self, screen_pos: Vec2) -> IVec2 {
        // Convert screen coordinates to tile coordinates based on the internal rendering resolution.
        let world_pos = self.screen_wc(screen_pos);
        IVec2::new(
            ((world_pos.x / self.dims.x as f32 * self.dims.x as f32) / TILE_SIZE) as i32,
            ((world_pos.y / self.dims.y as f32 * self.dims.y as f32) / TILE_SIZE) as i32,
        )
    }

    pub fn wc_screen(&self, world_pos: Vec2) -> Vec2 {
        // Convert world coordinates to screen coordinates using the camera's zoom and offset.
        let cam_target = Vec2::new(self.camera.target.x, self.camera.target.y);
        let cam_offset = Vec2::new(self.camera.offset.x, self.camera.offset.y);
        (world_pos - cam_target) * self.camera.zoom + cam_offset
    }
}

pub fn center_window(rl: &mut RaylibHandle, width: i32, height: i32) {
    // Get the index of the monitor the window is currently on.
    let monitor = get_current_monitor();

    // Get the dimensions and position of that monitor.
    let monitor_width = get_monitor_width(monitor);
    let monitor_height = get_monitor_height(monitor);
    let monitor_pos = get_monitor_position(monitor);

    // Safely get the monitor's name by matching on the Result.
    let monitor_name = match get_monitor_name(monitor) {
        Ok(name) => name,
        Err(_) => "N/A".to_string(),
    };

    // Print some useful monitor info for debugging.
    println!(
        "Centering on Monitor {}: '{}' ({}x{}) at ({}, {})",
        monitor, monitor_name, monitor_width, monitor_height, monitor_pos.x, monitor_pos.y
    );

    // Calculate the top-left position for the window to be centered on the current monitor.
    let x = monitor_pos.x as i32 + (monitor_width - width) / 2;
    let y = monitor_pos.y as i32 + (monitor_height - height) / 2;

    // Set the window's new position.
    rl.set_window_position(x, y);
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
