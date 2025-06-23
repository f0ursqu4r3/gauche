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

/// Spawns clouds that drift slowly across the screen from left to right.
/// This function should be called every frame.
///
/// # Arguments
/// * `state` - The main game state, used to access particles and camera info.
/// * `graphics` - The graphics context, needed to determine screen boundaries.
/// * `cloud_density` - A float from 0.0 (no clouds) to 1.0 (heavy clouds) that
///                   determines the probability of a new cloud spawning each frame.
pub fn spawn_weather_clouds(
    state: &mut State,
    graphics: &crate::graphics::Graphics,
    cloud_density: f32,
) {
    // --- Cloud Effect :: Tunable Parameters ---
    const MIN_CLOUD_SPEED: f32 = 0.005;
    const MAX_CLOUD_SPEED: f32 = 0.015;
    const MIN_CLOUD_SIZE: f32 = 64.0;
    const MAX_CLOUD_SIZE: f32 = 256.0;
    const MIN_CLOUD_ALPHA: f32 = 0.05;
    const MAX_CLOUD_ALPHA: f32 = 0.1;
    // A small probability multiplier to make the 0-1 density value feel right.
    const SPAWN_CHANCE_SCALAR: f32 = 0.02;

    // Determine if a new cloud should spawn on this frame
    if random_range(0.0..1.0) > cloud_density * SPAWN_CHANCE_SCALAR {
        return;
    }

    // --- Calculate Spawn Position & Lifetime ---
    // Get the camera's view boundaries in world coordinates
    let top_left_world = graphics.screen_to_world(Vec2::ZERO);
    let bottom_right_world = graphics.screen_to_world(graphics.window_dims.as_vec2());
    let screen_width_world = bottom_right_world.x - top_left_world.x;

    // Spawn the cloud just off-screen to the left
    let spawn_x = top_left_world.x - (MAX_CLOUD_SIZE / 16.0); // Convert pixel size to tile units
    let spawn_y = random_range(top_left_world.y..bottom_right_world.y);
    let spawn_pos = Vec2::new(spawn_x, spawn_y);

    let speed = random_range(MIN_CLOUD_SPEED..=MAX_CLOUD_SPEED);
    // Lifetime is the time it takes to cross the screen plus its own width
    let lifetime_in_frames = ((screen_width_world + (MAX_CLOUD_SIZE / 16.0)) / speed) as u32;

    // --- Create the Particle ---
    let sprite = match random_range(0..3) {
        0 => Sprite::Cloud1,
        1 => Sprite::Cloud2,
        _ => Sprite::Cloud3,
    };

    let size = random_range(MIN_CLOUD_SIZE..=MAX_CLOUD_SIZE);
    let alpha = random_range(MIN_CLOUD_ALPHA..=MAX_CLOUD_ALPHA);

    let particle_data = ParticleData::new(
        spawn_pos,
        Vec2::splat(size),
        0.0, // No rotation
        alpha,
        lifetime_in_frames,
        sprite,
        ParticleLayer::Paralaxing { height: 50 },
    );

    // Clouds have a simple, constant velocity, so we use `spawn_dynamic`.
    state
        .particles
        .spawn_dynamic(particle_data, Vec2::new(speed, 0.0), 0.0);
}

/// Spawns a generic debris effect using the provided sprite.
/// Good for tile damage, things breaking, etc.
pub fn debris_splatter(
    particles: &mut Particles,
    spawn_pos: Vec2,
    base_direction: Vec2,
    debris_sprite: Sprite,
) {
    // --- Debris Splatter :: Tunable Parameters ---
    const CONE_ANGLE: f32 = 90.0;
    const GRAVITY: f32 = 0.018;
    const NUM_PARTICLES: u32 = 8;
    const MIN_SIZE: f32 = 1.0;
    const MAX_SIZE: f32 = 4.0;
    const MIN_SPEED: f32 = 0.04;
    const MAX_SPEED: f32 = 0.1;
    const MIN_LIFETIME: u32 = 8;
    const MAX_LIFETIME: u32 = 20;

    for _ in 0..NUM_PARTICLES {
        let size = random_range(MIN_SIZE..=MAX_SIZE);
        let initial_speed = random_range(MIN_SPEED..=MAX_SPEED);
        let lifetime = random_range(MIN_LIFETIME..=MAX_LIFETIME);

        let angle_offset = random_range(-CONE_ANGLE..=CONE_ANGLE);
        let direction_rad = base_direction.y.atan2(base_direction.x) + angle_offset.to_radians();
        let final_direction = Vec2::new(direction_rad.cos(), direction_rad.sin());

        let vel = final_direction * initial_speed;
        let acc = Vec2::new(0.0, GRAVITY);

        let particle_data = ParticleData::new(
            spawn_pos,
            Vec2::splat(size),
            random_range(0.0..360.0), // Give it a random rotation
            1.0,
            lifetime,
            debris_sprite,
            ParticleLayer::Foreground,
        );

        particles.spawn_accelerated(particle_data, vel, acc);
    }
}
