use crate::{
    audio::{Audio, SoundEffect},
    entity_behavior::calc_sound_loudness_from_player_dist_falloff,
    particle::{ParticleData, ParticleLayer, Particles},
    sprite::Sprite,
    state::State,
    step::FRAMES_PER_SECOND,
};
use glam::Vec2;
use rand::random_range;

/// Spawns a complete blood splatter effect, including particles and sound, scaled by intensity.
/// All behavioral parameters are controlled by the constants at the top of this file for easy tuning.
pub fn blood_splatter(
    state: &mut State,
    audio: &mut Audio,
    spawn_pos: Vec2,
    base_direction: Vec2,
    magnitude: f32,
) {
    // --- Blood Splatter Effect :: Tunable Parameters ---
    const BLOOD_SPLATTER_SOUND: SoundEffect = SoundEffect::ZombieScratch1;
    const BLOOD_SPLATTER_CONE_ANGLE: f32 = 60.0; // The width of the spray cone in degrees (+/- this value)
    const BLOOD_SPLATTER_GRAVITY: f32 = 0.015; // A stronger downward pull

    // --- Magnitude-Scaled Parameters ---
    const BLOOD_SPLATTER_BASE_PARTICLES: f32 = 4.0;
    const BLOOD_SPLATTER_MAX_PARTICLES: f32 = 40.0;
    const BLOOD_SPLATTER_MAGNITUDE_SCALAR: f32 = 35.0;

    // --- Randomized Parameters (within a range) ---
    const BLOOD_SPLATTER_MIN_SIZE: f32 = 2.0;
    const BLOOD_SPLATTER_MAX_SIZE: f32 = 7.0;
    const BLOOD_SPLATTER_MIN_SPEED: f32 = 0.05; // Faster particles
    const BLOOD_SPLATTER_MAX_SPEED: f32 = 0.12;
    const BLOOD_SPLATTER_MIN_LIFETIME: u32 = 5; // Shorter lifetime
    const BLOOD_SPLATTER_MAX_LIFETIME: u32 = 15;

    // --- 1. Play Sound Effect ---
    let loudness = calc_sound_loudness_from_player_dist_falloff(state, spawn_pos, 16.0);
    if loudness > 0.0 {
        let final_volume = loudness * magnitude.clamp(0.5, 1.5);
        audio.play_sound_effect_scaled(BLOOD_SPLATTER_SOUND, final_volume);
    }

    // --- 2. Calculate Particle Count based on Magnitude ---
    let num_particles =
        (BLOOD_SPLATTER_BASE_PARTICLES + BLOOD_SPLATTER_MAGNITUDE_SCALAR * magnitude)
            .round()
            .clamp(BLOOD_SPLATTER_BASE_PARTICLES, BLOOD_SPLATTER_MAX_PARTICLES) as u32;

    // --- 3. Spawn Particles in a Loop ---
    for _ in 0..num_particles {
        let sprite = if random_range(0.0..1.0) < 0.8 {
            Sprite::BloodSmall
        } else {
            Sprite::BloodMedium
        };

        let size = random_range(BLOOD_SPLATTER_MIN_SIZE..=BLOOD_SPLATTER_MAX_SIZE);
        let initial_speed = random_range(BLOOD_SPLATTER_MIN_SPEED..=BLOOD_SPLATTER_MAX_SPEED);
        let lifetime = random_range(BLOOD_SPLATTER_MIN_LIFETIME..=BLOOD_SPLATTER_MAX_LIFETIME);

        let angle_offset = random_range(-BLOOD_SPLATTER_CONE_ANGLE..=BLOOD_SPLATTER_CONE_ANGLE);
        let direction_rad = base_direction.y.atan2(base_direction.x) + angle_offset.to_radians();
        let final_direction = Vec2::new(direction_rad.cos(), direction_rad.sin());

        let vel = final_direction * initial_speed * (1.0 + magnitude);
        let acc = Vec2::new(0.0, BLOOD_SPLATTER_GRAVITY);

        let particle_data = ParticleData::new(
            spawn_pos,
            Vec2::splat(size),
            0.0,
            1.0,
            lifetime,
            sprite,
            ParticleLayer::Foreground,
        );

        state.particles.spawn_accelerated(particle_data, vel, acc);
    }
}

/// Spawns a long-lasting, static puddle of blood on the ground.
/// This should be called right after `blood_splatter` to complete the effect.
pub fn blood_puddle(particles: &mut Particles, spawn_pos: Vec2, magnitude: f32) {
    // --- Blood Puddle Effect :: Tunable Parameters (NEW!) ---
    const BLOOD_PUDDLE_LIFETIME_SECONDS: u32 = 60;
    const BLOOD_PUDDLE_BASE_PARTICLES: f32 = 2.0;
    const BLOOD_PUDDLE_MAX_PARTICLES: f32 = 8.0;
    const BLOOD_PUDDLE_MAGNITUDE_SCALAR: f32 = 6.0;
    const BLOOD_PUDDLE_MIN_SIZE: f32 = 2.0;
    const BLOOD_PUDDLE_MAX_SIZE: f32 = 6.0;
    const BLOOD_PUDDLE_ALPHA: f32 = 0.7;
    const BLOOD_PUDDLE_WIDTH_RADIUS: f32 = 0.6; // Horizontal spread in tile units
    const BLOOD_PUDDLE_HEIGHT_RADIUS: f32 = 0.2; // Vertical spread in tile units

    let num_particles = (BLOOD_PUDDLE_BASE_PARTICLES + BLOOD_PUDDLE_MAGNITUDE_SCALAR * magnitude)
        .round()
        .clamp(BLOOD_PUDDLE_BASE_PARTICLES, BLOOD_PUDDLE_MAX_PARTICLES)
        as u32;

    let lifetime = BLOOD_PUDDLE_LIFETIME_SECONDS * FRAMES_PER_SECOND;

    for _ in 0..num_particles {
        // More likely to be smaller puddles
        let sprite = if random_range(0.0..1.0) < 0.7 {
            Sprite::BloodSmall
        } else {
            Sprite::BloodMedium
        };

        // Create the wide oval shape by using different random ranges for X and Y
        let offset = Vec2::new(
            random_range(-BLOOD_PUDDLE_WIDTH_RADIUS..=BLOOD_PUDDLE_WIDTH_RADIUS),
            random_range(-BLOOD_PUDDLE_HEIGHT_RADIUS..=BLOOD_PUDDLE_HEIGHT_RADIUS),
        );
        let final_pos = spawn_pos + offset;

        let size = random_range(BLOOD_PUDDLE_MIN_SIZE..=BLOOD_PUDDLE_MAX_SIZE);
        let rotation = random_range(0.0..360.0);

        let particle_data = ParticleData::new(
            final_pos,
            Vec2::splat(size),
            rotation,
            BLOOD_PUDDLE_ALPHA,
            lifetime,
            sprite,
            ParticleLayer::Background, // Render on the ground, behind entities
        );

        particles.spawn_static(particle_data);
    }
}
