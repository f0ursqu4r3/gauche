use glam::Vec2;

use crate::{entity::DrawLayer, special_effect_getters};

use super::special_effect::{get_sample_region, SampleRegion, SpecialEffect, SpecialEffectType};

pub struct StaticEffect {
    pub type_: SpecialEffectType,
    pub counter: u32,
    pub draw_layer: DrawLayer,

    pub pos: Vec2,
    pub size: Vec2,
    pub rot: f32,
    pub alpha: f32,
}

impl SpecialEffect for StaticEffect {
    special_effect_getters!();

    fn step(&mut self) {
        if self.counter > 0 {
            self.counter -= 1;
        }
    }
}
