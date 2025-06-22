// src/particle.rs

use crate::{graphics::Graphics, sprite::Sprite, state::State};
use glam::Vec2;
use raylib::prelude::{Color, RaylibDraw, RaylibDrawHandle, RaylibTextureMode, Rectangle, Vector2};

// --- 1. Common Data & Specific Particle Structs ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleLayer {
    Background, // Renders below tiles and entities (e.g., footprints, ground effects)
    Foreground, // Renders above everything (e.g., weather, hit effects)
}

/// Contains common data shared by all particle types.
/// This keeps the memory layout consistent and simplifies access.
#[derive(Debug, Clone)]
pub struct ParticleData {
    pub pos: Vec2,
    pub size: Vec2,
    pub rot: f32,
    pub alpha: f32,
    pub initial_alpha: f32, // Initial alpha value, used to calculate fade over lifetime
    pub lifetime: u32,
    pub initial_lifetime: u32, // Initial lifetime, used to calculate fade over lifetime
    pub sprite: Sprite,
    pub layer: ParticleLayer, // Layer to control rendering order
}

impl ParticleData {
    /// A convenient constructor for creating particle data.
    /// Automatically sets `initial_alpha` and `initial_lifetime` from the given values.
    pub fn new(
        pos: Vec2,
        size: Vec2,
        rot: f32,
        alpha: f32,
        lifetime: u32,
        sprite: Sprite,
        layer: ParticleLayer,
    ) -> Self {
        Self {
            pos,
            size,
            rot,
            alpha,                      // This will be updated by the step function
            initial_alpha: alpha,       // Set from the single `alpha` parameter
            lifetime,                   // This will be updated by the step function
            initial_lifetime: lifetime, // Set from the single `lifetime` parameter
            sprite,
            layer,
        }
    }
}

/// A particle that does not move.
#[derive(Debug, Clone)]
pub struct StaticParticle {
    pub data: ParticleData,
}

/// A particle that moves with a constant velocity.
#[derive(Debug, Clone)]
pub struct DynamicParticle {
    pub data: ParticleData,
    pub vel: Vec2,
    pub rot_vel: f32, // Optional: add rotational velocity
}

/// A particle that moves with velocity and acceleration.
#[derive(Debug, Clone)]
pub struct AcceleratedParticle {
    pub data: ParticleData,
    pub vel: Vec2,
    pub acc: Vec2,
}

/// A particle that follows a quadratic Bezier curve.
#[derive(Debug, Clone)]
pub struct SplineParticle {
    pub data: ParticleData,
    pub start_pos: Vec2,
    pub control_point: Vec2,
    pub end_pos: Vec2,
}

/// A particle that cycles through a list of sprites over its lifetime.
#[derive(Debug, Clone)]
pub struct AnimatedParticle {
    pub data: ParticleData,
    pub vel: Vec2, // Can also have velocity
    pub animation_sprites: Vec<Sprite>,
}

// --- 2. The Particle Manager ---

/// Manages all active particles in the game.
/// It holds a separate vector for each particle type to ensure a tight memory layout
/// and avoid wasted space, while providing excellent cache performance for updates.
#[derive(Debug, Clone)]
pub struct Particles {
    pub static_particles: Vec<StaticParticle>,
    pub dynamic_particles: Vec<DynamicParticle>,
    pub accelerated_particles: Vec<AcceleratedParticle>,
    pub spline_particles: Vec<SplineParticle>,
    pub animated_particles: Vec<AnimatedParticle>,
}

impl Particles {
    /// Creates a new, empty particle manager.
    pub fn new() -> Self {
        Self {
            static_particles: Vec::new(),
            dynamic_particles: Vec::new(),
            accelerated_particles: Vec::new(),
            spline_particles: Vec::new(),
            animated_particles: Vec::new(),
        }
    }

    /// Updates the state of all particles and removes any that have expired.
    /// This should be called once per game tick.
    pub fn step(&mut self) {
        // --- Update particle states ---

        for p in &mut self.static_particles {
            p.data.lifetime = p.data.lifetime.saturating_sub(1);
            let lifetime_ratio = p.data.lifetime as f32 / p.data.initial_lifetime as f32;
            p.data.alpha = p.data.initial_alpha * lifetime_ratio;
        }

        for p in &mut self.dynamic_particles {
            p.data.pos += p.vel;
            p.data.rot += p.rot_vel;
            p.data.lifetime = p.data.lifetime.saturating_sub(1);
            let lifetime_ratio = p.data.lifetime as f32 / p.data.initial_lifetime as f32;
            p.data.alpha = p.data.initial_alpha * lifetime_ratio;
        }

        for p in &mut self.accelerated_particles {
            p.vel += p.acc;
            p.data.pos += p.vel;
            p.data.lifetime = p.data.lifetime.saturating_sub(1);
            let lifetime_ratio = p.data.lifetime as f32 / p.data.initial_lifetime as f32;
            p.data.alpha = p.data.initial_alpha * lifetime_ratio;
        }

        for p in &mut self.spline_particles {
            let age_ratio = 1.0 - (p.data.lifetime as f32) / (p.data.initial_lifetime as f32);
            p.data.pos = calculate_bezier_point(age_ratio, p.start_pos, p.control_point, p.end_pos);
            p.data.lifetime = p.data.lifetime.saturating_sub(1);
            let lifetime_ratio = p.data.lifetime as f32 / p.data.initial_lifetime as f32;
            p.data.alpha = p.data.initial_alpha * lifetime_ratio;
        }

        for p in &mut self.animated_particles {
            p.data.pos += p.vel;
            p.data.lifetime = p.data.lifetime.saturating_sub(1);
            let lifetime_ratio = p.data.lifetime as f32 / p.data.initial_lifetime as f32;
            p.data.alpha = p.data.initial_alpha * lifetime_ratio;

            // Update sprite based on age
            let num_frames = p.animation_sprites.len();
            if num_frames > 0 {
                let age_ratio = 1.0 - (p.data.lifetime as f32) / (p.data.initial_lifetime as f32);
                let current_frame = ((age_ratio * num_frames as f32) as usize).min(num_frames - 1);
                p.data.sprite = p.animation_sprites[current_frame];
            }
        }

        // --- Clean up expired particles ---
        self.static_particles.retain(|p| p.data.lifetime > 0);
        self.dynamic_particles.retain(|p| p.data.lifetime > 0);
        self.accelerated_particles.retain(|p| p.data.lifetime > 0);
        self.spline_particles.retain(|p| p.data.lifetime > 0);
        self.animated_particles.retain(|p| p.data.lifetime > 0);
    }

    // --- 3. Spawner Functions (Public API) ---

    /// Spawns a non-moving particle.
    pub fn spawn_static(&mut self, data: ParticleData) {
        self.static_particles.push(StaticParticle { data });
    }

    /// Spawns a particle with constant velocity.
    pub fn spawn_dynamic(&mut self, data: ParticleData, vel: Vec2, rot_vel: f32) {
        self.dynamic_particles
            .push(DynamicParticle { data, vel, rot_vel });
    }

    /// Spawns a particle with acceleration.
    pub fn spawn_accelerated(&mut self, data: ParticleData, vel: Vec2, acc: Vec2) {
        self.accelerated_particles
            .push(AcceleratedParticle { data, vel, acc });
    }

    /// Spawns a particle that follows a curve.
    pub fn spawn_spline(
        &mut self,
        data: ParticleData,
        start_pos: Vec2,
        control_point: Vec2,
        end_pos: Vec2,
    ) {
        self.spline_particles.push(SplineParticle {
            data,
            start_pos,
            control_point,
            end_pos,
        });
    }

    /// Spawns a particle that plays an animation.
    pub fn spawn_animated(
        &mut self,
        data: ParticleData,
        vel: Vec2,
        animation_sprites: Vec<Sprite>,
    ) {
        self.animated_particles.push(AnimatedParticle {
            data,
            vel,
            animation_sprites,
        });
    }

    /// Removes all particles of all types.
    pub fn clear(&mut self) {
        self.static_particles.clear();
        self.dynamic_particles.clear();
        self.accelerated_particles.clear();
        self.spline_particles.clear();
        self.animated_particles.clear();
    }
}

// --- 4. Helper Functions ---

/// Calculates a point on a quadratic Bezier curve.
/// t is the progress along the curve, from 0.0 to 1.0.
fn calculate_bezier_point(t: f32, p0: Vec2, p1: Vec2, p2: Vec2) -> Vec2 {
    let t = t.clamp(0.0, 1.0);
    let one_minus_t = 1.0 - t;
    p0 * one_minus_t.powi(2) + p1 * 2.0 * one_minus_t * t + p2 * t.powi(2)
}

// These functions are kept private to the particle module.
// They take a generic slice `&[T]` so they are reusable.
fn draw_particle_slice<T>(
    d: &mut RaylibTextureMode<RaylibDrawHandle>,
    graphics: &Graphics,
    particles: &[T],
    get_data: impl Fn(&T) -> &ParticleData,
    target_layer: ParticleLayer, // The layer we want to draw
) {
    for p in particles {
        let data = get_data(p);
        // This check ensures we only draw particles for the correct layer.
        if data.lifetime > 0 && data.layer == target_layer {
            if let Some(texture) = graphics.get_sprite_texture(data.sprite) {
                let source_rec =
                    Rectangle::new(0.0, 0.0, texture.width as f32, texture.height as f32);
                let dest_rec = Rectangle::new(
                    data.pos.x * 16.0,
                    data.pos.y * 16.0,
                    data.size.x,
                    data.size.y,
                );
                let origin = Vector2::new(data.size.x / 2.0, data.size.y / 2.0);
                d.draw_texture_pro(
                    texture,
                    source_rec,
                    dest_rec,
                    origin,
                    data.rot,
                    Color::new(255, 255, 255, (data.alpha * 255.0) as u8),
                );
            }
        }
    }
}

fn draw_static_particles(
    d: &mut RaylibTextureMode<RaylibDrawHandle>,
    graphics: &Graphics,
    particles: &[StaticParticle],
    layer: ParticleLayer,
) {
    draw_particle_slice(d, graphics, particles, |p| &p.data, layer);
}

fn draw_dynamic_particles(
    d: &mut RaylibTextureMode<RaylibDrawHandle>,
    graphics: &Graphics,
    particles: &[DynamicParticle],
    layer: ParticleLayer,
) {
    draw_particle_slice(d, graphics, particles, |p| &p.data, layer);
}

fn draw_accelerated_particles(
    d: &mut RaylibTextureMode<RaylibDrawHandle>,
    graphics: &Graphics,
    particles: &[AcceleratedParticle],
    layer: ParticleLayer,
) {
    draw_particle_slice(d, graphics, particles, |p| &p.data, layer);
}

fn draw_spline_particles(
    d: &mut RaylibTextureMode<RaylibDrawHandle>,
    graphics: &Graphics,
    particles: &[SplineParticle],
    layer: ParticleLayer,
) {
    draw_particle_slice(d, graphics, particles, |p| &p.data, layer);
}

fn draw_animated_particles(
    d: &mut RaylibTextureMode<RaylibDrawHandle>,
    graphics: &Graphics,
    particles: &[AnimatedParticle],
    layer: ParticleLayer,
) {
    draw_particle_slice(d, graphics, particles, |p| &p.data, layer);
}

pub fn render_particles(
    d: &mut RaylibTextureMode<RaylibDrawHandle>,
    state: &State,
    graphics: &Graphics,
    layer: ParticleLayer,
) {
    draw_static_particles(d, graphics, &state.particles.static_particles, layer);
    draw_dynamic_particles(d, graphics, &state.particles.dynamic_particles, layer);
    draw_accelerated_particles(d, graphics, &state.particles.accelerated_particles, layer);
    draw_spline_particles(d, graphics, &state.particles.spline_particles, layer);
    draw_animated_particles(d, graphics, &state.particles.animated_particles, layer);
}
