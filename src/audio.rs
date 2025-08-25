use std::sync::OnceLock;
use raylib::core::audio::{RaylibAudio, Sound, Music};
use crate::types::WeaponType;


// Dispositivo global (una sola instancia)
static AUDIO_DEV: OnceLock<RaylibAudio> = OnceLock::new();

fn device() -> &'static RaylibAudio {
    AUDIO_DEV.get_or_init(|| {
        let ra = RaylibAudio::init_audio_device()
            .expect("No se pudo iniciar el dispositivo de audio");
        ra.set_master_volume(1.0);
        ra
    })
}

pub struct Audio {
    ra: &'static RaylibAudio,
    // --- SFX existentes ---
    pub snd_consume:  Sound<'static>,
    pub snd_pistol:   Sound<'static>,
    pub snd_smg:      Sound<'static>,
    pub snd_rifle:    Sound<'static>,
    pub snd_shotgun:  Sound<'static>,
    pub snd_rocket:   Sound<'static>,
    pub snd_reload_pistol:   Sound<'static>,
    pub snd_reload_smg:      Sound<'static>,
    pub snd_reload_rifle:    Sound<'static>,
    pub snd_reload_shotgun:  Sound<'static>,
    pub snd_reload_rocket:   Sound<'static>,
    // --- NUEVO: sonidos del jugador ---
    pub snd_player_hurt:  Sound<'static>,
    pub snd_player_death: Sound<'static>,

    pub snd_enemy_hurt:  Sound<'static>,
    pub snd_enemy_death: Sound<'static>,

    pub music_game: Music<'static>,
}

impl Audio {
    pub fn new() -> Self {
        let ra = device();

        // Ajusta rutas si usas otras
        let snd_consume = ra.new_sound("assets/sfx/consume.wav").expect("Falta consume.wav");

        let snd_pistol  = ra.new_sound("assets/sfx/weapons/pistol.wav").expect("Falta pistol.wav");
        let snd_smg     = ra.new_sound("assets/sfx/weapons/smg.wav").expect("Falta smg.wav");
        let snd_rifle   = ra.new_sound("assets/sfx/weapons/rifle.wav").expect("Falta rifle.wav");
        let snd_shotgun = ra.new_sound("assets/sfx/weapons/shotgun.wav").expect("Falta shotgun.wav");
        let snd_rocket  = ra.new_sound("assets/sfx/weapons/rocket.wav").expect("Falta rocket.wav");

        let snd_reload_pistol  = ra.new_sound("assets/sfx/weapons/reload_pistol.wav").expect("Falta reload_pistol.wav");
        let snd_reload_smg     = ra.new_sound("assets/sfx/weapons/reload_smg.wav").expect("Falta reload_smg.wav");
        let snd_reload_rifle   = ra.new_sound("assets/sfx/weapons/reload_rifle.wav").expect("Falta reload_rifle.wav");
        let snd_reload_shotgun = ra.new_sound("assets/sfx/weapons/reload_shotgun.wav").expect("Falta reload_shotgun.wav");
        let snd_reload_rocket  = ra.new_sound("assets/sfx/weapons/reload_rocket.wav").expect("Falta reload_rocket.wav");

        // 🔊 NUEVO: daño/muerte del jugador
        let snd_player_hurt  = ra.new_sound("assets/sfx/hurt.wav").expect("Falta player/hurt.wav");
        let snd_player_death = ra.new_sound("assets/sfx/death.wav").expect("Falta player/death.wav");

        let snd_enemy_hurt  = ra.new_sound("assets/sfx/enemy_hurt.wav").expect("Falta enemy/hurt.wav");
        let snd_enemy_death = ra.new_sound("assets/sfx/enemy_death.wav").expect("Falta enemy/death.wav");

         // 🎵 Música (loop manual)
        let music_game = ra.new_music("assets/sfx/audio/level_theme.mp3")
            .expect("Falta assets/sfx/audio/level_theme.mp3");
        music_game.set_volume(0.6);
        music_game.play_stream();

        Self {
            ra,
            snd_consume,
            snd_pistol, snd_smg, snd_rifle, snd_shotgun, snd_rocket,
            snd_reload_pistol, snd_reload_smg, snd_reload_rifle, snd_reload_shotgun, snd_reload_rocket,
            snd_player_hurt, snd_player_death, snd_enemy_hurt,
            snd_enemy_death, music_game,
        }
    }

    pub fn set_master(&self, v: f32) { self.ra.set_master_volume(v.clamp(0.0, 1.0)); }
    pub fn play_consume(&self) { self.snd_consume.play(); }
    pub fn play_shot(&self, wtype: WeaponType) {
        match wtype {
            WeaponType::Pistol => self.snd_pistol.play(),
            WeaponType::SMG => self.snd_smg.play(),
            WeaponType::Rifle => self.snd_rifle.play(),
            WeaponType::Shotgun => self.snd_shotgun.play(),
            WeaponType::RocketLauncher => self.snd_rocket.play(),
        }
    }
    pub fn play_reload(&self, wtype: WeaponType) {
        match wtype {
            WeaponType::Pistol => self.snd_reload_pistol.play(),
            WeaponType::SMG => self.snd_reload_smg.play(),
            WeaponType::Rifle => self.snd_reload_rifle.play(),
            WeaponType::Shotgun => self.snd_reload_shotgun.play(),
            WeaponType::RocketLauncher => self.snd_reload_rocket.play(),
        }
    }
    // --- NUEVO: hooks de daño del jugador ---
    pub fn play_player_hurt(&self)  { self.snd_player_hurt.play(); }
    pub fn play_player_death(&self) { self.snd_player_death.play(); }
    pub fn play_enemy_hurt(&self)  { self.snd_enemy_hurt.play(); }
    pub fn play_enemy_death(&self) { self.snd_enemy_death.play(); }
        pub fn update(&self) {
        self.music_game.update_stream();
        // loop manual
        let len = self.music_game.get_time_length();
        let pos = self.music_game.get_time_played();
        if pos >= len - 0.05 {
            self.music_game.seek_stream(0.0);
        }
    }

    pub fn music_pause(&self)  { self.music_game.pause_stream(); }
pub fn music_resume(&self) { self.music_game.resume_stream(); }
pub fn music_set_volume(&self, v: f32) { self.music_game.set_volume(v.clamp(0.0, 1.0)); }
}
