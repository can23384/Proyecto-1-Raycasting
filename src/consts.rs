use raylib::prelude::*;

pub const SCREEN_W: i32 = 1280;
pub const SCREEN_H: i32 = 720;

pub const PLAYER_MAX_HP: i32 = 100;
pub const PLAYER_MAX_SHIELD: i32 = 100;

pub const SMALL_HEALTH_CAP: i32 = 75;
pub const SMALL_SHIELD_CAP: i32 = 50;

// 🔹 Cantidades por pickup de munición
pub const AMMO_LIGHT_PACK: i32  = 30;
pub const AMMO_MEDIUM_PACK: i32 = 20;
pub const AMMO_HEAVY_PACK: i32  = 10;
pub const AMMO_SHELL_PACK: i32  = 8;
pub const AMMO_ROCKET_PACK: i32 = 2;

// Máximos de stack por consumible
pub const MAX_STACK_HEALTH_SMALL: i32 = 15;
pub const MAX_STACK_HEALTH_BIG:   i32 = 3;
pub const MAX_STACK_SHIELD_SMALL: i32 = 6;
pub const MAX_STACK_SHIELD_BIG:   i32 = 3;

// Cooldowns (segundos) por consumible
pub const CD_HEALTH_SMALL: f32 = 2.5;
pub const CD_HEALTH_BIG:   f32 = 10.0;
pub const CD_SHIELD_SMALL: f32 = 2.5;
pub const CD_SHIELD_BIG:   f32 = 3.5;


// Probabilidades (0.0–1.0). Deben sumar 1.0 por tipo.
pub const P_HEALTH_NONE:  f32 = 0.40;
pub const P_HEALTH_SMALL: f32 = 0.40; // Vida menor (20 hasta 75)
pub const P_HEALTH_BIG:   f32 = 0.20; // Vida mayor (+100 hasta 100)

pub const P_SHIELD_NONE:  f32 = 0.40;
pub const P_SHIELD_SMALL: f32 = 0.40; // Escudo menor (+25 hasta 50)
pub const P_SHIELD_BIG:   f32 = 0.20; // Escudo mayor (+50 hasta 100)


// Probabilidad de que NO haya arma en un spawn (0.0–1.0)
pub const P_WEAPON_NONE: f32 = 0.40; // 40% no spawnea nada

// Distribución de rarezas (suma 1.0)
pub const P_RARITY_COMMON:     f32 = 0.45;
pub const P_RARITY_UNCOMMON:   f32 = 0.25;
pub const P_RARITY_RARE:       f32 = 0.18;
pub const P_RARITY_EPIC:       f32 = 0.09;
pub const P_RARITY_LEGENDARY:  f32 = 0.03;


// ===== Melee / Puño =====
pub const PUNCH_DAMAGE: i32   = 25;   // daño del golpe
pub const PUNCH_RANGE: f32    = 1.4;  // alcance efectivo (metros/celdas)
pub const PUNCH_COOLDOWN: f32 = 0.6;  // tiempo entre golpes (s)

// --- Spawns aleatorios de munición ---
pub const P_AMMO_NONE:   f32 = 0.35; // no aparece nada
pub const P_AMMO_LIGHT:  f32 = 0.20; // balas ligeras
pub const P_AMMO_MED:    f32 = 0.18; // balas medianas
pub const P_AMMO_HEAVY:  f32 = 0.12; // balas pesadas
pub const P_AMMO_SHELL:  f32 = 0.10; // cartuchos escopeta
pub const P_AMMO_ROCKET: f32 = 0.05; // cohetes
