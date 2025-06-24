use raylib::prelude::*;
use std::collections::HashMap;
use strum::{EnumIter, IntoEnumIterator, IntoStaticStr}; // Use EnumIter for iteration, IntoStaticStr for auto-filenames.

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, EnumIter, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Song {
    Title,
    Playing,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, EnumIter, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum SoundEffect {
    ApeScream,
    BallBounce1,
    BallBounce2,
    BallBounce3,
    BallBounce4,
    BallDrop,
    BallHitPaddle,
    BallHitPaddleEr,
    BallWallBounce,
    Confirm,
    Death,
    Explosion,
    Explosion1,
    Explosion2,
    Explosion3,
    AnimalCrush1,
    AnimalCrush2,
    HitBlock1,
    LevelLoss,
    LevelStart,
    LevelWin,
    SmallLaser,
    SturdyBlockBouncedOn,
    SuperConfirm,
    Step1,
    Step2,
    BoxBreak,
    BlockLand,
    ZombieGrowl1,
    ZombieGrowl2,
    ZombieScratch1,
    Punch1,
    ClothRip,
    CantUse,
}

pub const SOUND_EFFECT_COOLDOWN: f32 = 0.1;

/// The main struct for managing all game audio. It holds the loaded songs and sounds.
pub struct Audio<'a> {
    pub current_song: Option<Song>,
    pub songs: HashMap<Song, Music<'a>>,
    pub sounds: HashMap<SoundEffect, Sound<'a>>,
    pub music_volume: f32,
    pub sound_effects_volume: f32,

    pub sound_effect_cooldowns: HashMap<SoundEffect, f32>,
}

impl<'a> Audio<'a> {
    /// Creates a new `Audio` instance, loading all songs and sounds from disk.
    pub fn new(rl_audio: &'a RaylibAudio) -> Result<Audio<'a>, String> {
        Ok(Self {
            current_song: None,
            songs: load_songs(rl_audio)?,
            sounds: load_sounds(rl_audio)?,
            music_volume: 1.0,
            sound_effects_volume: 1.0,
            sound_effect_cooldowns: HashMap::new(),
        })
    }

    /// Plays a song from the `Song` enum, stopping any previously playing song.
    pub fn play_song(&mut self, song: Song) {
        if self.current_song == Some(song) {
            return; // Don't restart if it's the same song.
        }
        self.stop_current_song(); // Stop whatever is currently playing.

        self.current_song = Some(song);
        if let Some(music) = self.songs.get_mut(&song) {
            music.set_volume(self.music_volume);
            music.play_stream();
        }
    }

    /// Stops the currently playing song, if any.
    pub fn stop_current_song(&mut self) {
        if let Some(current_song) = self.current_song {
            // We call the private helper function.
            self.stop_song(current_song);
            self.current_song = None;
        }
    }

    /// Private helper to stop a specific song.
    fn stop_song(&mut self, song: Song) {
        if let Some(music) = self.songs.get_mut(&song) {
            music.stop_stream();
        }
    }

    /// Updates the buffer for the currently streaming song. Must be called every frame.
    pub fn update_current_song_stream_data(&mut self) {
        if let Some(song) = self.current_song {
            if let Some(music) = self.songs.get_mut(&song) {
                music.update_stream();
            }
        }
    }

    /// Plays a one-shot sound effect from the `SoundEffect` enum.
    pub fn play_sound_effect(&mut self, sound_effect: SoundEffect) {
        // Check if the sound effect is on cooldown.
        if let Some(cooldown) = self.sound_effect_cooldowns.get(&sound_effect) {
            if *cooldown > 0.0 {
                // If the cooldown is still active, do not play the sound.
                return;
            }
        }

        if let Some(sound) = self.sounds.get_mut(&sound_effect) {
            sound.set_volume(self.sound_effects_volume);
            sound.play();

            // Reset the cooldown for this sound effect.
            self.sound_effect_cooldowns
                .insert(sound_effect, SOUND_EFFECT_COOLDOWN);
        }
    }

    /// Plays a sound effect with a volume scaled by the provided factor.
    /// The final volume will be `master_sfx_volume * scale`.
    /// The scale factor will be clamped between 0.0 and 1.0.
    pub fn play_sound_effect_scaled(&mut self, sound_effect: SoundEffect, scale: f32) {
        // Check if the sound effect is on cooldown.
        if let Some(cooldown) = self.sound_effect_cooldowns.get(&sound_effect) {
            if *cooldown > 0.0 {
                // If the cooldown is still active, do not play the sound.
                return;
            }
        }

        if let Some(sound) = self.sounds.get_mut(&sound_effect) {
            // Calculate the final volume by multiplying the master sound effects volume by the scale factor.
            let final_volume = self.sound_effects_volume * scale.clamp(0.0, 1.0);
            sound.set_volume(final_volume);
            sound.play();
        }

        // Reset the cooldown for this sound effect.
        self.sound_effect_cooldowns
            .insert(sound_effect, SOUND_EFFECT_COOLDOWN);
    }

    /// Sets the volume for all music tracks and updates the currently playing one.
    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = volume.clamp(0.0, 1.0);
        if let Some(song) = self.current_song {
            if let Some(music) = self.songs.get_mut(&song) {
                music.set_volume(self.music_volume);
            }
        }
    }

    /// Sets the volume for all sound effects.
    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.sound_effects_volume = volume.clamp(0.0, 1.0);
    }

    pub fn step_sound_effect_cooldowns(&mut self, delta_time: f32) {
        // Update cooldowns for all sound effects.
        for cooldown in self.sound_effect_cooldowns.values_mut() {
            if *cooldown > 0.0 {
                *cooldown -= delta_time;
                if *cooldown < 0.0 {
                    *cooldown = 0.0; // Ensure it doesn't go negative.
                }
            }
        }

        // Remove any sound effects that have no cooldown left.
        self.sound_effect_cooldowns
            .retain(|_, &mut cooldown| cooldown > 0.0);
    }
}

// --- Asset Loading ---

/// Loads all `Song` variants from the `assets/music` directory.
pub fn load_songs(rl_audio: &RaylibAudio) -> Result<HashMap<Song, Music<'_>>, String> {
    let mut songs = HashMap::new();
    println!("--- Loading Music ---");
    for song in Song::iter() {
        let filename: &'static str = song.into(); // Strum magic!
        let path = format!("assets/music/{}.ogg", filename);

        match rl_audio.new_music(&path) {
            Ok(music) => {
                println!("- Loaded: {}", path);
                songs.insert(song, music);
            }
            Err(e) => return Err(format!("Failed to load music '{}': {}", path, e)),
        }
    }
    println!("--- {} music tracks loaded. ---", songs.len());
    Ok(songs)
}

/// Loads all `SoundEffect` variants from the `assets/sounds` directory.
/// Note the added lifetime `'a` on the `RaylibAudio` reference.
pub fn load_sounds(rl_audio: &RaylibAudio) -> Result<HashMap<SoundEffect, Sound<'_>>, String> {
    let mut sounds = HashMap::new();
    println!("--- Loading Sound Effects ---");
    for sound_effect in SoundEffect::iter() {
        let filename: &'static str = sound_effect.into(); // Strum magic!
        let path = format!("assets/sounds/{}.ogg", filename);

        match rl_audio.new_sound(&path) {
            Ok(sound) => {
                println!("- Loaded: {}", path);
                sounds.insert(sound_effect, sound);
            }
            Err(e) => return Err(format!("Failed to load sound '{}': {}", path, e)),
        }
    }
    println!("--- {} sound effects loaded. ---", sounds.len());
    Ok(sounds)
}
