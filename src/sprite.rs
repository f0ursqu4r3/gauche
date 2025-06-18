use std::{fs::File, io::BufReader, path::Path};

use glam::{UVec2, Vec2};
use rand::Rng;
use strum::{EnumCount, EnumIter, IntoEnumIterator};

#[derive(Copy, Clone, Debug, EnumIter, EnumCount, PartialEq, Eq, Hash)]
pub enum Sprite {
    NoSprite,
    Reticle,
    GrassTile,

    PlayerNeutral,
    PlayerDead,
}

impl Sprite {
    pub fn to_filename(self) -> &'static str {
        match self {
            Sprite::NoSprite => "no_sprite",
            Sprite::Reticle => "reticle",
            Sprite::GrassTile => "grass_tile",

            // player
            Sprite::PlayerNeutral => "player_neutral",
            Sprite::PlayerDead => "player_dead",
        }
    }
}

#[derive(Debug)]
pub struct SpriteData {
    pub size: UVec2,
}
