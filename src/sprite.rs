use std::{fs::File, io::BufReader, path::Path};

use glam::{UVec2, Vec2};
use rand::Rng;
use strum::{EnumCount, EnumIter, IntoEnumIterator};

#[derive(Copy, Clone, Debug, EnumIter, EnumCount, PartialEq, Eq, Hash)]
pub enum Sprite {
    NoSprite,
    Reticle,

    PlayerNeutral,
    PlayerDead,
}

impl Sprite {
    pub fn to_filename(self) -> &'static str {
        match self {
            Sprite::NoSprite => "no_sprite",
            Sprite::Reticle => "reticle",

            // player
            Sprite::PlayerNeutral => "player_neutral",
            Sprite::PlayerDead => "player_dead",
        }
    }
}

#[derive(Debug)]
pub struct SpriteData {
    pub sample_position: UVec2,
    pub size: UVec2,
}

pub fn load_sprites(asset_folder: &str) -> Result<Vec<SpriteData>, String> {
    // before we load them lets just check all the files are there

    let mut missing_files = vec![];
    for sprite in Sprite::iter() {
        let filename = sprite.to_filename();
        let png_path = Path::new(asset_folder).join(format!("{}.png", filename));
        let json_path = Path::new(asset_folder).join(format!("{}.json", filename));
        if !png_path.exists() {
            missing_files.push(png_path.to_str().unwrap().to_string());
        }
        if !json_path.exists() {
            missing_files.push(json_path.to_str().unwrap().to_string());
        }
    }

    if !missing_files.is_empty() {
        // build up a big error string with newlines and tabs
        // make it print in red
        println!("\x1b[31m");
        println!("Missing files:");
        for file in missing_files.iter() {
            println!("\t{}", file);
        }
        println!("\x1b[0m");
        return Err(format!("Missing files: {:?}", missing_files));
    }

    let mut sprites: Vec<SpriteData> = vec![];
    for sprite in Sprite::iter() {
        let filename = sprite.to_filename();
        let json_path = Path::new(asset_folder).join(format!("{}.json", filename));
        let sprite_data = load_sprite_data(&json_path)?;
        sprites.push(sprite_data);
    }
    Ok(sprites)
}

// pub fn load_sprite_data(json_path: &Path) -> Result<SpriteData, String> {
//     let file = File::open(json_path).map_err(|e| format!("Failed to open JSON file: {}", e))?;
//     let reader = BufReader::new(file);
//     let json: Value =
//         serde_json::from_reader(reader).map_err(|e| format!("Failed to parse JSON: {}", e))?;

//     let frames = json["frames"].as_object().ok_or("Invalid JSON structure")?;
//     let mut sprite_frames = Vec::new();
//     let mut size = UVec2::ZERO;

//     for (_, frame_data) in frames {
//         let frame = frame_data["frame"]
//             .as_object()
//             .ok_or("Invalid frame data")?;
//         let x = frame["x"].as_u64().ok_or("Invalid x")? as u32;
//         let y = frame["y"].as_u64().ok_or("Invalid y")? as u32;
//         let duration = frame_data["duration"].as_f64().ok_or("Invalid duration")? as f32;

//         sprite_frames.push(Frame {
//             sample_position: UVec2::new(x, y),
//             duration,
//         });

//         if size == UVec2::ZERO {
//             let w = frame["w"].as_u64().ok_or("Invalid w")? as u32;
//             let h = frame["h"].as_u64().ok_or("Invalid h")? as u32;
//             size = UVec2::new(w, h);
//         }
//     }

//     Ok(SpriteData {
//         frames: sprite_frames,
//         size,
//     })
// }
