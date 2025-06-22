use glam::Vec2;

use crate::{entity::DrawLayer, special_effect_getters};

use super::special_effect::{get_sample_region, SampleRegion, SpecialEffect, SpecialEffectType};

pub struct UltraDynamicEffect {
    pub type_: SpecialEffectType,
    pub counter: u32,
    pub draw_layer: DrawLayer,

    pub pos: Vec2,
    pub size: Vec2,
    pub rot: f32,
    pub alpha: f32,

    pub vel: Vec2,
    pub svel: Vec2,
    pub rotvel: f32,
    pub alpha_vel: f32,

    pub acc: Vec2,
    pub sacc: Vec2,
    pub rotacc: f32,
    pub alpha_acc: f32,
}

impl SpecialEffect for UltraDynamicEffect {
    special_effect_getters!();

    fn step(&mut self) {
        if self.counter > 0 {
            self.counter -= 1;
        }

        self.vel += self.acc;
        self.svel += self.sacc;
        self.rotvel += self.rotacc;
        self.alpha_vel += self.alpha_acc;

        self.pos += self.vel;
        self.size += self.svel;
        self.rot += self.rotvel;
        self.alpha += self.alpha_vel;

        self.size = self.size.max(Vec2::ZERO);
        self.alpha = self.alpha.min(1.0).max(0.0);
    }
}
