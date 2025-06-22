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

    /// Converts window/screen coordinates to pixel-based WORLD coordinates.
    /// This is the known-good function from our working test example.
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        // 1. Scale window mouse pos to render texture pos
        let scale = self.dims.as_vec2() / self.window_dims.as_vec2();
        let texture_pos = screen_pos * scale;

        // 2. Manually perform the inverse camera transform using raw math.
        let cam_target = Vec2::new(self.camera.target.x, self.camera.target.y);
        let cam_offset = Vec2::new(self.camera.offset.x, self.camera.offset.y);

        let pos = (texture_pos - cam_offset) / self.camera.zoom + cam_target;
        // divide by tile_size
        pos / TILE_SIZE
    }

    pub fn screen_to_tile(&self, screen_pos: Vec2) -> Vec2 {
        // Convert screen coordinates to world coordinates
        let world_pos = self.screen_to_world(screen_pos);
        // Convert world coordinates to tile coordinates
        Vec2::new(world_pos.x.floor(), world_pos.y.floor())
    }

    /// Converts world coordinates back to window/screen coordinates.
    /// This is the exact inverse of the `screen_to_world` function.
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        // The input `world_pos` is in world tile units. First, convert it to world pixel units.
        let world_pixel_pos = world_pos * TILE_SIZE;

        // Now, perform the forward camera transformation to get the position in render texture space.
        let cam_target = Vec2::new(self.camera.target.x, self.camera.target.y);
        let cam_offset = Vec2::new(self.camera.offset.x, self.camera.offset.y);
        let texture_pos = (world_pixel_pos - cam_target) * self.camera.zoom + cam_offset;

        // Finally, scale from the render texture space to the window space.
        let scale = self.window_dims.as_vec2() / self.dims.as_vec2();
        texture_pos * scale
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

fn window_to_tile_coords(
    window_pos: Vec2,
    window_dims: Vec2,
    render_dims: Vec2,
    camera: Camera2D,
) -> Vec2 {
    // Step 1: Scale the mouse coordinate from Window Space to Render Texture Space.
    let scale = render_dims / window_dims;
    let texture_coord = window_pos * scale;

    // Step 2: Manually reverse the camera transformation.
    // This is the algebraic inverse of the transformation that happens when you call `begin_mode2D`.
    let cam_target = Vec2::new(camera.target.x, camera.target.y);
    let cam_offset = Vec2::new(camera.offset.x, camera.offset.y);

    // (texture_coord - offset) -> Reverses the offset translation.
    // (... / zoom)             -> Reverses the zoom scaling.
    // (... + target)           -> Reverses the target translation.
    (texture_coord - cam_offset) / camera.zoom + cam_target
}
