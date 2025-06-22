use glam::Vec2;

use crate::{entity::DrawLayer, special_effect_getters};

use super::special_effect::{get_sample_region, SampleRegion, SpecialEffect, SpecialEffectType};

pub struct SplineEffect {
    pub type_: SpecialEffectType,
    pub counter: u32,
    pub draw_layer: DrawLayer,

    pub point_1: Vec2,
    pub point_2: Vec2,
    pub point_3: Vec2,

    pub t: f32,
    pub pos: Vec2,
    pub size: Vec2,
    pub rot: f32,
    pub alpha: f32,

    pub tvel: Vec2,
    pub svel: Vec2,
    pub rotvel: f32,
    pub alpha_vel: f32,

    pub tacc: Vec2,
    pub sacc: Vec2,
    pub rotacc: f32,
    pub alpha_acc: f32,
}

impl SpecialEffect for SplineEffect {
    special_effect_getters!();

    fn step(&mut self) {
        if self.counter > 0 {
            self.counter -= 1;
        }

        self.tvel += self.tacc;
        self.svel += self.sacc;
        self.rotvel += self.rotacc;
        self.alpha_vel += self.alpha_acc;

        self.t += self.tvel.x; // TODO, this cant be right
        self.size += self.svel;
        self.rot += self.rotvel;
        self.alpha += self.alpha_vel;

        self.size = self.size.max(Vec2::ZERO);
        self.alpha = self.alpha.min(1.0).max(0.0);

        self.t = self.t.min(1.0).max(0.0);
        let new_pos = calculate_bezier_point(self.t, self.point_1, self.point_2, self.point_3);
        self.pos = new_pos;
    }
}

fn calculate_bezier_point(t: f32, point_1: Vec2, point_2: Vec2, point_3: Vec2) -> Vec2 {
    let one_minus_t = 1.0 - t;
    (point_1 * one_minus_t * one_minus_t) + (point_2 * 2.0 * one_minus_t * t) + (point_3 * t * t)
}
