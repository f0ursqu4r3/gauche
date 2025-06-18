use raylib::prelude::*;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Song {
    Title,
    Playing,
}

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Hash)]
pub enum SoundEffect {
    ApeScream,
    BallBounce1,
    BallBounce2,
    BallBounce3,
    BallBounce4,
    BallDrop,
    BallHitPaddleEr,
    BallWallBounce,
    Confirm,
    Death,
    Explosion,
    Explosion1,
    Explosion2,
    Explosion3,
    HitBlock1,
    LevelLoss,
    LevelStart,
    LevelWin,
    SmallLaser,
    SturdyBlockBouncedOn,
    SuperConfirm,
}

pub struct Audio<'a> {
    pub current_song: Option<Song>,
    pub songs: Vec<Music<'a>>,
    pub sounds: Vec<Sound<'a>>,
    pub music_volume: f32,
    pub sound_effects_volume: f32,
}

pub fn load_songs(rl_audio: &RaylibAudio) -> Vec<Music> {
    let mut songs = Vec::new();
    let file_names = vec!["title", "playing"];
    for name in file_names {
        let path = format!("assets/music/{}.ogg", name);
        let music = match rl_audio.new_music(path.as_str()) {
            Ok(music) => music,
            Err(e) => {
                panic!("Error loading music: {}", e);
            }
        };
        songs.push(music);
    }
    songs
}

pub fn load_sounds(rl_audio: &RaylibAudio) -> Vec<Sound> {
    let mut sounds = Vec::new();

    for sound_effect in SoundEffect::iter() {
        let file_name_prefix = get_sound_file_name(sound_effect);
        let path = format!("assets/sounds/{}.ogg", file_name_prefix);
        let sound = match rl_audio.new_sound(path.as_str()) {
            Ok(sound) => sound,
            Err(e) => {
                panic!("Error loading sound: {}", e);
            }
        };
        sounds.push(sound);
    }
    sounds
}

impl<'a> Audio<'a> {
    pub fn new(songs: Vec<Music<'a>>, sounds: Vec<Sound<'a>>) -> Audio<'a> {
        Self {
            current_song: None,
            songs,
            sounds,
            music_volume: 1.0,
            sound_effects_volume: 1.0,
        }
    }

    pub fn play_song(&mut self, song: Song) {
        // stop current song if its different from the new song
        if let Some(current_song) = self.current_song {
            if current_song != song {
                self.stop_song(current_song);
            }
        }

        self.current_song = Some(song);

        let song = &mut self.songs[song as usize];
        song.set_volume(self.music_volume);
        song.play_stream();
    }

    pub fn stop_current_song(&mut self) {
        if let Some(current_song) = self.current_song {
            self.stop_song(current_song);
        }
    }

    fn stop_song(&mut self, song: Song) {
        let song = &mut self.songs[song as usize];
        song.stop_stream();
    }

    pub fn update_current_song_stream_data(&mut self) {
        if let Some(song) = self.current_song {
            let song = &mut self.songs[song as usize];
            song.update_stream();
        }
    }

    pub fn play_sound_effect(&mut self, sound_effect: SoundEffect) {
        let sound_effect = &mut self.sounds[sound_effect as usize];
        sound_effect.set_volume(self.sound_effects_volume);
        sound_effect.play();
    }

    pub fn set_current_song_volume(&mut self, volume: f32) {
        if let Some(song) = self.current_song {
            let song = &mut self.songs[song as usize];
            song.set_volume(volume);
        }
    }
}

pub fn get_sound_file_name(se: SoundEffect) -> &'static str {
    match se {
        SoundEffect::ApeScream => "ape_scream",
        SoundEffect::BallBounce1 => "ball_bounce_1",
        SoundEffect::BallBounce2 => "ball_bounce_2",
        SoundEffect::BallBounce3 => "ball_bounce_3",
        SoundEffect::BallBounce4 => "ball_bounce_4",
        SoundEffect::BallDrop => "ball_drop",
        SoundEffect::BallHitPaddleEr => "ball_hit_paddle_er",
        SoundEffect::BallWallBounce => "ball_wall_bounce",
        SoundEffect::Confirm => "confirm",
        SoundEffect::Death => "death",
        SoundEffect::Explosion => "explosion",
        SoundEffect::Explosion1 => "explosion_1",
        SoundEffect::Explosion2 => "explosion_2",
        SoundEffect::Explosion3 => "explosion_3",
        SoundEffect::HitBlock1 => "hit_block_1",
        SoundEffect::LevelLoss => "level_loss",
        SoundEffect::LevelStart => "level_start",
        SoundEffect::LevelWin => "level_win",
        SoundEffect::SmallLaser => "small_laser",
        SoundEffect::SturdyBlockBouncedOn => "sturdy_block_bounced_on",
        SoundEffect::SuperConfirm => "super_confirm",
    }
}
