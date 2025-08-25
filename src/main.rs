mod consts;
mod types;
mod player;
mod ai;
mod render;
mod map;
mod hud;
mod audio;
use audio::Audio;
mod menu; // ⬅️ arriba junto a tus otros mod
use menu::{Menu, MenuAction};
mod victory; // nuevo
use victory::{VictoryScreen, VictoryAction};





use raylib::core::audio::{RaylibAudio, Music};

use raylib::prelude::*;
use rand::{thread_rng, Rng};
use rand::seq::SliceRandom;

use crate::consts::*;

use types::{
    Player, Enemy, EnemyState, Weapon, WeaponType, Rarity,
    AmmoType, Pickup, PickupKind,
    Decoration, DecoKind, Chest,
    Item, SlotItem, ConsumableType, WeaponState,  
};
use player::{handle_input, MoveCfg};
use ai::{update_enemies, AiCfg};
use render::{draw_frame, RenderParams};
use map::Map;
use hud::WeaponHudTextures;




#[derive(Clone, Copy, PartialEq, Eq)]
enum GameState { Menu, Playing, Victory }






// Empieza a usar: sólo arma el temporizador si es válido usar ahora.
fn start_use_consumable(player: &Player, slot: &mut SlotItem) -> bool {
    if slot.using || slot.count <= 0 { return false; }
    match slot.item {
        Item::Consumable(ConsumableType::HealthSmall) => {
            if player.hp >= SMALL_HEALTH_CAP { return false; }
            slot.using = true;
            slot.cd = CD_HEALTH_SMALL; // 2.5s
            true
        }
        Item::Consumable(ConsumableType::HealthBig) => {
            slot.using = true;
            slot.cd = CD_HEALTH_BIG;   // 10s
            true
        }
        Item::Consumable(ConsumableType::ShieldSmall) => {
            if player.shield >= SMALL_SHIELD_CAP { return false; }
            slot.using = true;
            slot.cd = CD_SHIELD_SMALL; // 2.5s
            true
        }
        Item::Consumable(ConsumableType::ShieldBig) => {
            slot.using = true;
            slot.cd = CD_SHIELD_BIG;   // 3.5s
            true
        }
        Item::Weapon(_, _) => false,
    }
}

// Aplica el efecto cuando termina la canalización (cd <= 0)
fn finish_use_consumable(player: &mut Player, slot: &mut SlotItem) {
    if !slot.using || slot.count <= 0 { return; }
    match slot.item {
        Item::Consumable(ConsumableType::HealthSmall) => {
            // aplica ahora
            player.hp = (player.hp + 20).min(SMALL_HEALTH_CAP);
            slot.count -= 1;
        }
        Item::Consumable(ConsumableType::HealthBig) => {
            player.hp = (player.hp + 100).min(PLAYER_MAX_HP);
            slot.count -= 1;
        }
        Item::Consumable(ConsumableType::ShieldSmall) => {
            player.shield = (player.shield + 25).min(SMALL_SHIELD_CAP);
            slot.count -= 1;
        }
        Item::Consumable(ConsumableType::ShieldBig) => {
            player.shield = (player.shield + 50).min(PLAYER_MAX_SHIELD);
            slot.count -= 1;
        }
        Item::Weapon(_, _) => { /* no aplica */ }
    }
    // termina el uso
    slot.using = false;
    slot.cd = 0.0; // listo para iniciar otro uso cuando quieras
}






// ---------------- Helpers sin préstamos conflictivos ----------------

fn make_dropped_from_slot(px: f32, py: f32, slot: &SlotItem) -> Vec<Pickup> {
    let mut rng = thread_rng();
    let mut out = Vec::new();
    match slot.item {
        Item::Weapon(w, _ws) => {
            out.push(Pickup {
                x: px + rng.gen_range(-0.25..0.25),
                y: py + rng.gen_range(-0.25..0.25),
                kind: PickupKind::Weapon { wtype: w.wtype, rarity: w.rarity },
                radius: 0.35,
                color: w.rarity.color(),
                spawn_lock: 0.35,
            });
        }
        Item::Consumable(c) => {
            let (color, kind) = match c {
                ConsumableType::HealthSmall => (Color::LIME,    PickupKind::HealthSmall),
                ConsumableType::HealthBig   => (Color::GOLD,    PickupKind::HealthBig),
                ConsumableType::ShieldSmall => (Color::SKYBLUE, PickupKind::ShieldSmall),
                ConsumableType::ShieldBig   => (Color::BLUE,    PickupKind::ShieldBig),
            };
            for _ in 0..slot.count.max(0) {
                out.push(Pickup {
                    x: px + rng.gen_range(-0.25..0.25),
                    y: py + rng.gen_range(-0.25..0.25),
                    kind,
                    radius: 0.35,
                    color,
                    spawn_lock: 0.35,
                });
            }
        }
    }
    out
}

fn max_stack_and_cd(c: ConsumableType) -> (i32, f32) {
    match c {
        ConsumableType::HealthSmall => (MAX_STACK_HEALTH_SMALL, CD_HEALTH_SMALL),
        ConsumableType::HealthBig   => (MAX_STACK_HEALTH_BIG,   CD_HEALTH_BIG),
        ConsumableType::ShieldSmall => (MAX_STACK_SHIELD_SMALL, CD_SHIELD_SMALL),
        ConsumableType::ShieldBig   => (MAX_STACK_SHIELD_BIG,   CD_SHIELD_BIG),
    }
}

struct SlotApplyResult { consumed: bool, drops: Vec<Pickup> }

fn try_stack_or_replace_slot(
    player: &mut Player,
    mut new_item: SlotItem, // recuerda poner new_item.using = false al crear
    selected_slot: Option<usize>,
) -> SlotApplyResult {
    let mut res = SlotApplyResult { consumed: false, drops: Vec::new() };
    let Some(si) = selected_slot else { return res; };

    if player.slots[si].is_none() {
        player.slots[si] = Some(new_item);
        res.consumed = true;
        return res;
    }

    // Intentar apilar si son el mismo consumible (conserva 'using' y el mayor cd)
    if let Some(slot) = &mut player.slots[si] {
        if let (Item::Consumable(a), Item::Consumable(b)) = (slot.item, new_item.item) {
            if a == b {
                let (max_stack, _) = max_stack_and_cd(b);
                let free = (max_stack - slot.count).max(0);
                if free > 0 {
                    // conserva el estado de uso y el mayor cd
                    slot.cd = slot.cd.max(new_item.cd);
                    // slot.using se mantiene como estaba (si estaba usando, sigue)
                    slot.count += new_item.count.min(free);
                    res.consumed = true;
                    return res;
                } else {
                    return res;
                }
            }
        }
    }

    // Reemplazo: si cambia de tipo, cancelar uso
    // En la rama de reemplazo de try_stack_or_replace_slot:
    if let Some(old) = &player.slots[si] {
        res.drops = make_dropped_from_slot(player.x, player.y, old);
    }

    // Si el nuevo ítem es diferente, entra limpio (sin recarga/uso)
    match &mut new_item.item {
        Item::Consumable(_) => {
            new_item.using = false;
            new_item.cd = 0.0;
        }
        Item::Weapon(_, ws_new) => {
            ws_new.reloading = false;
            ws_new.reload_cd = 0.0;
            // opcional:
            // ws_new.weapon_cd = 0.0;
        }
    }

    player.slots[si] = Some(new_item);
    res.consumed = true;
    return res;

}


fn roll_weapon_type<R: Rng>(rng: &mut R) -> WeaponType {
    use WeaponType::*;
    let all = [Pistol, SMG, Rifle, Shotgun, RocketLauncher];
    *all.choose(rng).unwrap()
}

fn roll_rarity<R: Rng>(rng: &mut R) -> Rarity {
    let t: f32 = rng.gen_range(0.0..1.0); // usa gen_range para evitar el conflicto con gen en 2024 edition
    let c  = crate::consts::P_RARITY_COMMON;
    let uc = c  + crate::consts::P_RARITY_UNCOMMON;
    let rr = uc + crate::consts::P_RARITY_RARE;
    let ep = rr + crate::consts::P_RARITY_EPIC;
    if t < c  { Rarity::Common }
    else if t < uc { Rarity::Uncommon }
    else if t < rr { Rarity::Rare }
    else if t < ep { Rarity::Epic }
    else { Rarity::Legendary }
}




// --------------------------- MAIN ---------------------------

fn main() {

    let (mut rl, thread) = raylib::init()
        .size(SCREEN_W, SCREEN_H)
        .title("Raycasting - Slots unificados (armas + consumibles)")
        .build();
    rl.set_target_fps(60);

    let map = Map::from_txt("assets/map.txt").expect("No se pudo cargar assets/map.txt");

    let mut minimap = hud::Minimap::new();
    let mut elapsed: f32 = 0.0;
    let mut kills: u32 = 0;

    // Catálogo base de armas (rarezas por defecto Común)
    let weapons_catalog: Vec<Weapon> = vec![
        Weapon { wtype: WeaponType::Pistol, name: "Pistola", damage: 25, fire_interval: 0.35, mag_size: 12, reload_time: 1.2, ammo_type: AmmoType::Light, rarity: Rarity::Common },
        Weapon { wtype: WeaponType::SMG,    name: "SMG",     damage: 12, fire_interval: 0.08, mag_size: 30, reload_time: 1.6, ammo_type: AmmoType::Light, rarity: Rarity::Common },
        Weapon { wtype: WeaponType::Rifle,  name: "Rifle",   damage: 35, fire_interval: 0.50, mag_size: 10, reload_time: 2.0, ammo_type: AmmoType::Medium, rarity: Rarity::Common },
        Weapon { wtype: WeaponType::Shotgun,name: "Escopeta",damage: 50, fire_interval: 0.80, mag_size: 6,  reload_time: 1.8, ammo_type: AmmoType::Shell,  rarity: Rarity::Common },
        Weapon { wtype: WeaponType::RocketLauncher, name: "Lanzacohetes", damage: 120, fire_interval: 1.20, mag_size: 1, reload_time: 2.3, ammo_type: AmmoType::Rocket, rarity: Rarity::Common },
        
    ];

    // Estado inicial
    let (px, py) = map.player_spawn.unwrap_or((2.5, 2.5));
    let mut player = Player {
        x: px, y: py, angle: 0.0,
        hp: PLAYER_MAX_HP,
        shield: 0,
        ammo_reserve: [60, 40, 20, 12, 4],
        slots: [None, None, None, None, None],
        selected: None,
        punch_cd: 0.0,
    };

    // Arranca con pistola en slot 1
    if let Some(pistol) = weapons_catalog.iter().find(|w| w.wtype == WeaponType::Pistol) {
        let ws = WeaponState { ammo_in_mag: pistol.mag_size, weapon_cd: 0.0, reloading: false, reload_cd: 0.0 };
        player.slots[0] = Some(SlotItem { item: Item::Weapon(*pistol, ws), count: 1, cd: 0.0, using: false });
        player.selected = Some(0);
    }


    let mut rng = thread_rng();
    // Enemigos
    let mut enemies: Vec<Enemy> = map.enemy_spawns.iter().enumerate().map(|(i, &(ex, ey))| {
        let hp = rng.gen_range(100..=200);
         let weapon = *weapons_catalog
            .choose(&mut rng)
            .expect("weapons_catalog vacío");
        Enemy { x: ex, y: ey, hp, speed: 1.0,
                state: EnemyState::Idle, color: Color::ORANGE,
                weapon, weapon_cd: 0.0, flash_timer: 0.0 , death_anim_t: 0.0,}
    }).collect::<Vec<_>>();

    // Pickups iniciales desde mapa
    let mut pickups: Vec<Pickup> = Vec::new();

    use rand::{thread_rng, Rng};


// 🔹 Generar spawns aleatorios de VIDA

let mut rng = rand::thread_rng();

// VIDA
for &(x, y) in &map.heal_random_spawns {
    let r: f32 = rng.r#gen(); // <-- OJO: r#gen()
    if r < P_HEALTH_NONE {
        // nada
    } else if r < P_HEALTH_NONE + P_HEALTH_SMALL {
        pickups.push(Pickup { x, y, kind: PickupKind::HealthSmall, radius: 0.35, color: Color::LIME,    spawn_lock: 0.0 });
    } else {
        pickups.push(Pickup { x, y, kind: PickupKind::HealthBig,   radius: 0.35, color: Color::GOLD,    spawn_lock: 0.0 });
    }
}

// ESCUDO
for &(x, y) in &map.shield_random_spawns {
    let r: f32 = rng.r#gen(); // <-- r#gen() aquí también
    if r < P_SHIELD_NONE {
        // nada
    } else if r < P_SHIELD_NONE + P_SHIELD_SMALL {
        pickups.push(Pickup { x, y, kind: PickupKind::ShieldSmall, radius: 0.35, color: Color::SKYBLUE, spawn_lock: 0.0 });
    } else {
        pickups.push(Pickup { x, y, kind: PickupKind::ShieldBig,   radius: 0.35, color: Color::BLUE,    spawn_lock: 0.0 });
    }
}

// 🔹 Spawns aleatorios de ARMA
for &(x, y) in &map.weapon_random_spawns {
    // ¿aparece algo aquí?
    let r_none: f32 = rng.gen_range(0.0..1.0);
    if r_none < P_WEAPON_NONE {
        continue; // nada
    }

    // elige arma y rareza
    let wtype = roll_weapon_type(&mut rng);
    let rarity = roll_rarity(&mut rng);
    let color = rarity.color(); // ya lo usas para “brillo”/HUD

    pickups.push(Pickup {
        x, y,
        kind: PickupKind::Weapon { wtype, rarity },
        radius: 0.35,
        color,
        spawn_lock: 0.0,
    });
}

// 🔹 Generar spawns aleatorios de MUNICIÓN
for &(x, y) in &map.ammo_random_spawns {
    let r: f32 = rng.gen_range(0.0..1.0); // [0.0, 1.0)

    // Distribución por tramos
    let mut acc = P_AMMO_NONE;
    if r < acc {
        // No aparece nada
        continue;
    }
    acc += P_AMMO_LIGHT;
    if r < acc {
        pickups.push(Pickup {
            x, y,
            kind: PickupKind::Ammo { ammo: AmmoType::Light, amount: AMMO_LIGHT_PACK },
            radius: 0.35,
            color: Color::LIGHTGRAY, // si luego pones textura, este color no se verá
            spawn_lock: 0.0,
        });
        continue;
    }
    acc += P_AMMO_MED;
    if r < acc {
        pickups.push(Pickup {
            x, y,
            kind: PickupKind::Ammo { ammo: AmmoType::Medium, amount: AMMO_MEDIUM_PACK },
            radius: 0.35,
            color: Color::GRAY,
            spawn_lock: 0.0,
        });
        continue;
    }
    acc += P_AMMO_HEAVY;
    if r < acc {
        pickups.push(Pickup {
            x, y,
            kind: PickupKind::Ammo { ammo: AmmoType::Heavy, amount: AMMO_HEAVY_PACK },
            radius: 0.35,
            color: Color::DARKGRAY,
            spawn_lock: 0.0,
        });
        continue;
    }
    acc += P_AMMO_SHELL;
    if r < acc {
        pickups.push(Pickup {
            x, y,
            kind: PickupKind::Ammo { ammo: AmmoType::Shell, amount: AMMO_SHELL_PACK },
            radius: 0.35,
            color: Color::BROWN,
            spawn_lock: 0.0,
        });
        continue;
    }

    // Si no cayó en los anteriores, son cohetes
    pickups.push(Pickup {
        x, y,
        kind: PickupKind::Ammo { ammo: AmmoType::Rocket, amount: AMMO_ROCKET_PACK },
        radius: 0.35,
        color: Color::RED,
        spawn_lock: 0.0,
    });
}




    // Salud/Escudo
    pickups.extend(map.health_small_spawns.iter().map(|&(x,y)| Pickup { x, y, kind: PickupKind::HealthSmall, radius: 0.35, color: Color::LIME,    spawn_lock: 0.0 }));
    pickups.extend(map.health_big_spawns.iter().map(|&(x,y)|   Pickup { x, y, kind: PickupKind::HealthBig,   radius: 0.35, color: Color::GOLD,    spawn_lock: 0.0 }));
    pickups.extend(map.shield_small_spawns.iter().map(|&(x,y)| Pickup { x, y, kind: PickupKind::ShieldSmall, radius: 0.35, color: Color::SKYBLUE, spawn_lock: 0.0 }));
    pickups.extend(map.shield_big_spawns.iter().map(|&(x,y)|   Pickup { x, y, kind: PickupKind::ShieldBig,   radius: 0.35, color: Color::BLUE,    spawn_lock: 0.0 }));

    // Munición
    pickups.extend(map.ammo_light_spawns.iter().map(|&(x,y)|   Pickup { x, y, kind: PickupKind::Ammo{ ammo: AmmoType::Light,  amount: AMMO_LIGHT_PACK },  radius: 0.35, color: Color::WHITE,     spawn_lock: 0.0 }));
    pickups.extend(map.ammo_medium_spawns.iter().map(|&(x,y)|  Pickup { x, y, kind: PickupKind::Ammo{ ammo: AmmoType::Medium, amount: AMMO_MEDIUM_PACK }, radius: 0.35, color: Color::LIGHTGRAY, spawn_lock: 0.0 }));
    pickups.extend(map.ammo_heavy_spawns.iter().map(|&(x,y)|   Pickup { x, y, kind: PickupKind::Ammo{ ammo: AmmoType::Heavy,  amount: AMMO_HEAVY_PACK },  radius: 0.35, color: Color::DARKGRAY,  spawn_lock: 0.0 }));
    pickups.extend(map.ammo_shell_spawns.iter().map(|&(x,y)|   Pickup { x, y, kind: PickupKind::Ammo{ ammo: AmmoType::Shell,  amount: AMMO_SHELL_PACK },  radius: 0.35, color: Color::BROWN,     spawn_lock: 0.0 }));
    pickups.extend(map.ammo_rocket_spawns.iter().map(|&(x,y)|  Pickup { x, y, kind: PickupKind::Ammo{ ammo: AmmoType::Rocket, amount: AMMO_ROCKET_PACK }, radius: 0.35, color: Color::RED,       spawn_lock: 0.0 }));

    // Armas en suelo (A/M/R/O/K del mapa)
    let ground_rarity = Rarity::Common;
    let color_for = |r: Rarity| r.color();
    pickups.extend(map.weapon_pistol_spawns.iter().map(|&(x,y)|  Pickup { x, y, kind: PickupKind::Weapon{ wtype: WeaponType::Pistol,         rarity: ground_rarity }, radius: 0.35, color: color_for(ground_rarity), spawn_lock: 0.0 }));
    pickups.extend(map.weapon_smg_spawns.iter().map(|&(x,y)|     Pickup { x, y, kind: PickupKind::Weapon{ wtype: WeaponType::SMG,            rarity: ground_rarity }, radius: 0.35, color: color_for(ground_rarity), spawn_lock: 0.0 }));
    pickups.extend(map.weapon_rifle_spawns.iter().map(|&(x,y)|   Pickup { x, y, kind: PickupKind::Weapon{ wtype: WeaponType::Rifle,          rarity: ground_rarity }, radius: 0.35, color: color_for(ground_rarity), spawn_lock: 0.0 }));
    pickups.extend(map.weapon_shotgun_spawns.iter().map(|&(x,y)| Pickup { x, y, kind: PickupKind::Weapon{ wtype: WeaponType::Shotgun,        rarity: ground_rarity }, radius: 0.35, color: color_for(ground_rarity), spawn_lock: 0.0 }));
    pickups.extend(map.weapon_rocket_spawns.iter().map(|&(x,y)|  Pickup { x, y, kind: PickupKind::Weapon{ wtype: WeaponType::RocketLauncher, rarity: ground_rarity }, radius: 0.35, color: color_for(ground_rarity), spawn_lock: 0.0 }));

    // Decoraciones
    let mut decorations: Vec<Decoration> = Vec::new();
    for &(x, y) in &map.deco_block_spawns {
        decorations.push(Decoration { x, y, radius: 0.35, color: Color::DARKPURPLE, kind: DecoKind::Blocking });
    }
    for &(x, y) in &map.deco_ghost_spawns {
        decorations.push(Decoration { x, y, radius: 0.28, color: Color::LIGHTGRAY,  kind: DecoKind::Ghost });
    }

    // Cofres
    let mut chests: Vec<Chest> = map.chest_spawns.iter().map(|&(x,y)| Chest {
        x, y, radius: 0.33, opened: false, color_closed: Color::BROWN, color_opened: Color::GOLD
    }).collect::<Vec<_>>();

    // Cámara
    let fov: f32 = 60.0_f32.to_radians();
    let proj_dist: f32 = (SCREEN_W as f32 / 2.0) / (fov / 2.0).tan();

    // Movimiento + IA
    let move_cfg = MoveCfg { move_speed: 3.0, rot_speed: 2.5 };
    let ai_cfg = AiCfg { detect_radius: 6.0, melee_range: 0.0, melee_dps: 0.0, shoot_range: 7.0 };


let wall_paths = [
    "assets/walls/wall01.png", // id 1
    "assets/walls/wall02.png", // id 2
    "assets/walls/wall03.png", // id 3
    "assets/walls/wall04.png",
    "assets/walls/wall05.png",
    "assets/walls/wall06.png",
    "assets/walls/wall07.png",
    // añade más en el orden de sus ids...
];

let mut wall_textures: Vec<Texture2D> = Vec::new();
for p in wall_paths.iter() {
    let tex = rl.load_texture(&thread, p).expect("No se pudo cargar textura de pared");
    tex.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
    wall_textures.push(tex);
}


let tex_hp_small  = rl.load_texture(&thread, "assets/pickups/health_small.png") .expect("hp small tex");
let tex_hp_big    = rl.load_texture(&thread, "assets/pickups/health_big.png")   .expect("hp big tex");
let tex_sh_small  = rl.load_texture(&thread, "assets/pickups/shield_small.png") .expect("shield small tex");
let tex_sh_big    = rl.load_texture(&thread, "assets/pickups/shield_big.png")   .expect("shield big tex");

// 🔧 filtrado para que no se “pixelee” feo al escalar
for t in [&tex_hp_small, &tex_hp_big, &tex_sh_small, &tex_sh_big] {
    t.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
}


let tex_w_pistol  = rl.load_texture(&thread, "assets/pickups/pistol.png") .expect("pistol tex");
let tex_w_smg     = rl.load_texture(&thread, "assets/pickups/smg.png")    .expect("smg tex");
let tex_w_rifle   = rl.load_texture(&thread, "assets/pickups/rifle.png")  .expect("rifle tex");
let tex_w_shotgun = rl.load_texture(&thread, "assets/pickups/shotgun.png").expect("shotgun tex");
let tex_w_rocket  = rl.load_texture(&thread, "assets/pickups/rocket.png") .expect("rocket tex");

for t in [&tex_w_pistol, &tex_w_smg, &tex_w_rifle, &tex_w_shotgun, &tex_w_rocket] {
    t.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
}


let tex_ammo = rl
    .load_texture(&thread, "assets/pickups/ammo.png")
    .expect("ammo tex");
tex_ammo.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);


let tex_chest_closed = rl
    .load_texture(&thread, "assets/pickups/chest_closed.png")
    .expect("chest closed tex");
let tex_chest_opened = rl
    .load_texture(&thread, "assets/pickups/chest_open.png")
    .expect("chest opened tex");

for t in [&tex_chest_closed, &tex_chest_opened] {
    t.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
}


// Cargar texturas de enemigo (con expect para fallar con mensaje claro)
let tex_enemy_alive = rl
    .load_texture(&thread, "assets/enemies/enemy_alive.png")
    .expect("No se pudo cargar enemy_alive.png");

let tex_d0 = rl
    .load_texture(&thread, "assets/enemies/death_0.png")
    .expect("No se pudo cargar death_0.png");
let tex_d1 = rl
    .load_texture(&thread, "assets/enemies/death_1.png")
    .expect("No se pudo cargar death_1.png");
let tex_d2 = rl
    .load_texture(&thread, "assets/enemies/death_2.png")
    .expect("No se pudo cargar death_2.png");
let tex_d3 = rl
    .load_texture(&thread, "assets/enemies/death_3.png")
    .expect("No se pudo cargar death_3.png");

// Filtro (opcional)
for t in [&tex_enemy_alive, &tex_d0, &tex_d1, &tex_d2, &tex_d3] {
    t.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
}



let weapon_hud_tex = WeaponHudTextures {
    pistol:   rl.load_texture(&thread, "assets/hud/held_pistol.png").expect("falta held_pistol.png"),
    smg:      rl.load_texture(&thread, "assets/hud/held_smg.png").expect("falta held_smg.png"),
    rifle:    rl.load_texture(&thread, "assets/hud/held_rifle.png").expect("falta held_rifle.png"),
    shotgun:  rl.load_texture(&thread, "assets/hud/held_shotgun.png").expect("falta held_shotgun.png"),
    rocket:   rl.load_texture(&thread, "assets/hud/held_rocket.png").expect("falta held_rocket.png"),
};
  


let audio = Audio::new();

// Armar el paquete para el render
let enemy_tex = render::EnemyTextures {
    alive: Some(&tex_enemy_alive),
    death_frames: vec![&tex_d0, &tex_d1, &tex_d2, &tex_d3],
    death_frame_time: 0.08,
};


let chest_tex = render::ChestTextures {
    closed: Some(&tex_chest_closed),
    opened: Some(&tex_chest_opened),
};

let pickup_tex = render::PickupTextures {
    health_small: Some(&tex_hp_small),
    health_big:   Some(&tex_hp_big),
    shield_small: Some(&tex_sh_small),
    shield_big:   Some(&tex_sh_big),

    weapon_pistol:  Some(&tex_w_pistol),
    weapon_smg:     Some(&tex_w_smg),
    weapon_rifle:   Some(&tex_w_rifle),
    weapon_shotgun: Some(&tex_w_shotgun),
    weapon_rocket:  Some(&tex_w_rocket),
    ammo_generic: Some(&tex_ammo),
    
};

 
let mut state   = GameState::Menu;   // si no lo tenías
let mut menu    = Menu::new();       // ya lo usas
let mut victory = VictoryScreen::new();

    while !rl.window_should_close() {
    let dt = rl.get_frame_time();

    // ----- ESTADO: MENÚ -----


    // ----- ESTADO: MENÚ -----
    if state == GameState::Menu {
        match menu.handle_input(&rl) {
            MenuAction::Start => { state = GameState::Playing; }
            MenuAction::None => {}
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        menu.draw(&mut d);
        continue; // seguimos al siguiente frame del while
    }

    // ----- ESTADO: VICTORY -----
if state == GameState::Victory {
    // Entrada (fuera de begin_drawing)
    match victory.handle_input(&rl) {
        VictoryAction::ToMenu => {
            state = GameState::Menu;
            // Si quieres limpiar stats al volver al menú:
             elapsed = 0.0; kills = 0;
        }
        VictoryAction::Restart => {
            // Opción simple: también vuelve al menú (más seguro si aún no
            // extrajiste tu setup a una función).
            state = GameState::Menu;

            // Si ya tienes una función de setup del run (recomendada),
            // llámala aquí y pon Playing:
            // setup_run(&mut player, &mut enemies, &mut pickups, &mut decorations, &mut chests, &map, &weapons_catalog, &mut rng);
            // elapsed = 0.0; kills = 0;
            // state = GameState::Playing;
        }
        VictoryAction::None => {}
    }

    // Dibujo
    let mut d = rl.begin_drawing(&thread);
    d.clear_background(Color::BLACK);
    victory.draw(&mut d);
    continue;
}

        
        let dt = rl.get_frame_time();
        elapsed += dt;

        player.punch_cd = (player.punch_cd - dt).max(0.0);
        audio.update();






        // Bajar locks de pickups y cooldowns de slots
        for p in pickups.iter_mut() { p.spawn_lock = (p.spawn_lock - dt).max(0.0); }
        // 1) Degradar tiempos de todos los slots
// Degradar tiempos y aplicar efecto cuando termina la canalización
for i in 0..player.slots.len() {
    if let Some(mut slot) = player.slots[i].take() {
        if slot.using {
            slot.cd = (slot.cd - dt).max(0.0);
            if slot.cd <= 0.0 {
                // Ahora sí podemos mutar player y el slot al mismo tiempo,
                // porque el slot NO está dentro de player.slots (lo sacamos con take()).
                finish_use_consumable(&mut player, &mut slot);
            }
        }
        // decide si el slot sigue existiendo
        let keep = match slot.item {
            // si quieres que al llegar a 0 unidades se borre del slot:
            Item::Consumable(_) => slot.count > 0 || slot.using,
            Item::Weapon(_, _)  => true,
        };
        if keep {
            player.slots[i] = Some(slot);
        } // else: lo dejas como None (vacío)
    }
}



        // Movimiento (con colisiones con decoraciones/cofres)
        handle_input(&rl, &mut player, &move_cfg, dt, &map, &decorations, &chests);


        let prev_selected = player.selected;

         minimap.handle_input(&rl);

        // Selección de slots
        if rl.is_key_pressed(KeyboardKey::KEY_ZERO)  { player.selected = None; }
        if rl.is_key_pressed(KeyboardKey::KEY_ONE)   { player.selected = Some(0); }
        if rl.is_key_pressed(KeyboardKey::KEY_TWO)   { player.selected = Some(1); }
        if rl.is_key_pressed(KeyboardKey::KEY_THREE) { player.selected = Some(2); }
        if rl.is_key_pressed(KeyboardKey::KEY_FOUR)  { player.selected = Some(3); }
        if rl.is_key_pressed(KeyboardKey::KEY_FIVE)  { player.selected = Some(4); }

        if player.selected != prev_selected {
            if let Some(si) = prev_selected {
                if let Some(mut slot) = player.slots[si].take() {
                    match &mut slot.item {
                        // ❌ Consumible: ya lo cancelabas…
                        Item::Consumable(_) => {
                            slot.using = false;
                            slot.cd = 0.0;
                        }
                        // ❌ Arma: cancelar recarga en curso
                        Item::Weapon(_, ws) => {
                            if ws.reloading {
                                ws.reloading = false;
                                ws.reload_cd = 0.0; // pierde el progreso de recarga
                            }
                            // opcional: también puedes cancelar cadencia si quieres:
                            // ws.weapon_cd = 0.0;
                        }
                    }
                    player.slots[si] = Some(slot);
                }
            }
        }

        let fire_down        = rl.is_key_down(KeyboardKey::KEY_SPACE);
        let reload_pressed   = rl.is_key_pressed(KeyboardKey::KEY_R);
        let interact_pressed = rl.is_key_pressed(KeyboardKey::KEY_E);


        // Usar consumible (F) — sacar y devolver el slot para evitar doble préstamo
        // F (usar consumible)
if rl.is_key_pressed(KeyboardKey::KEY_F) {
    if let Some(si) = player.selected {
        if let Some(mut slot) = player.slots[si].take() {
            let started = start_use_consumable(&player, &mut slot);
            if started {
                audio.play_consume();

            }
            // Si el slot sigue existiendo (armas siempre, consumibles con count>0)
            let keep = match slot.item {
                Item::Consumable(_) => slot.count > 0 || slot.using,
                Item::Weapon(_, _)  => true,
            };
            if keep { player.slots[si] = Some(slot); }
        }
    }
}




        // Abrir cofres y soltar loot
        if interact_pressed {
            let r = 0.6_f32;
            let mut rng = thread_rng();

            for c in chests.iter_mut() {
                if c.opened { continue; }
                let dx = c.x - player.x;
                let dy = c.y - player.y;
                if (dx*dx + dy*dy).sqrt() <= r {
                    c.opened = true;

                    let tabla: [PickupKind; 14] = [
                        PickupKind::HealthSmall,
                        PickupKind::HealthBig,
                        PickupKind::ShieldSmall,
                        PickupKind::ShieldBig,
                        PickupKind::Ammo { ammo: AmmoType::Light,  amount: AMMO_LIGHT_PACK  },
                        PickupKind::Ammo { ammo: AmmoType::Medium, amount: AMMO_MEDIUM_PACK },
                        PickupKind::Ammo { ammo: AmmoType::Heavy,  amount: AMMO_HEAVY_PACK  },
                        PickupKind::Ammo { ammo: AmmoType::Shell,  amount: AMMO_SHELL_PACK  },
                        PickupKind::Ammo { ammo: AmmoType::Rocket, amount: AMMO_ROCKET_PACK },
                        PickupKind::Weapon { wtype: WeaponType::Pistol,         rarity: Rarity::Uncommon },
                        PickupKind::Weapon { wtype: WeaponType::SMG,            rarity: Rarity::Rare },
                        PickupKind::Weapon { wtype: WeaponType::Rifle,          rarity: Rarity::Rare },
                        PickupKind::Weapon { wtype: WeaponType::Shotgun,        rarity: Rarity::Epic },
                        PickupKind::Weapon { wtype: WeaponType::RocketLauncher, rarity: Rarity::Legendary },
                    ];

                    let num_drops = rng.gen_range(1..=2);
                    for kind in tabla.choose_multiple(&mut rng, num_drops) {
                        let (ox, oy) = (rng.gen_range(-0.3..0.3), rng.gen_range(-0.3..0.3));
                        let color = match kind {
                            PickupKind::HealthSmall => Color::LIME,
                            PickupKind::HealthBig   => Color::GOLD,
                            PickupKind::ShieldSmall => Color::SKYBLUE,
                            PickupKind::ShieldBig   => Color::BLUE,
                            PickupKind::Ammo{ ammo, .. } => match ammo {
                                AmmoType::Light  => Color::WHITE,
                                AmmoType::Medium => Color::LIGHTGRAY,
                                AmmoType::Heavy  => Color::DARKGRAY,
                                AmmoType::Shell  => Color::BROWN,
                                AmmoType::Rocket => Color::RED,
                            },
                            PickupKind::Weapon { rarity, .. } => rarity.color(),
                            PickupKind::Item { item, .. } => match item {
                                Item::Consumable(ConsumableType::HealthSmall) => Color::LIME,
                                Item::Consumable(ConsumableType::HealthBig)   => Color::GOLD,
                                Item::Consumable(ConsumableType::ShieldSmall) => Color::SKYBLUE,
                                Item::Consumable(ConsumableType::ShieldBig)   => Color::BLUE,
                                Item::Weapon(w, _) => w.rarity.color(),
                            },
                        };

                        pickups.push(Pickup {
                            x: c.x + ox, y: c.y + oy,
                            kind: *kind,
                            radius: 0.35,
                            color,
                            spawn_lock: 0.35,
                        });
                    }
                }
            }

            // Recoger pickups cercanos en 2 fases (no mutar durante la iteración)
            // Recoger pickups cercanos (al presionar E)
let mut to_remove = Vec::new();
let mut to_add = Vec::new();

// 🔹 Snapshot para evitar conflicto de préstamos
let selected_slot = player.selected;

for (i, p) in pickups.iter().enumerate() {
    if p.spawn_lock > 0.0 { continue; }
    let dx = p.x - player.x;
    let dy = p.y - player.y;
    if (dx*dx + dy*dy).sqrt() > 0.6 { continue; }

    match p.kind {
        // Munición → reserva
        PickupKind::Ammo { ammo, amount } => {
            let idx = ammo.index();
            player.ammo_reserve[idx] = player.ammo_reserve[idx].saturating_add(amount);
            to_remove.push(i);
        }

        // Consumibles unitarios → slot seleccionado
        PickupKind::HealthSmall => {
            let new_slot = SlotItem {
                item: Item::Consumable(ConsumableType::HealthSmall),
                count: 1,
                cd: 0.0, // <- ok; try_stack_or_replace_slot preserva el cd si corresponde
                using: false
            };
            let res = try_stack_or_replace_slot(&mut player, new_slot, selected_slot);
            if res.consumed { to_remove.push(i); }
            to_add.extend(res.drops);
        }
        PickupKind::HealthBig => {
            let new_slot = SlotItem { item: Item::Consumable(ConsumableType::HealthBig), count: 1, cd: 0.0, using: false };
            let res = try_stack_or_replace_slot(&mut player, new_slot, selected_slot);
            if res.consumed { to_remove.push(i); }
            to_add.extend(res.drops);
        }
        PickupKind::ShieldSmall => {
            let new_slot = SlotItem { item: Item::Consumable(ConsumableType::ShieldSmall), count: 1, cd: 0.0 , using: false};
            let res = try_stack_or_replace_slot(&mut player, new_slot, selected_slot);
            if res.consumed { to_remove.push(i); }
            to_add.extend(res.drops);
        }
        PickupKind::ShieldBig => {
            let new_slot = SlotItem { item: Item::Consumable(ConsumableType::ShieldBig), count: 1, cd: 0.0 , using: false};
            let res = try_stack_or_replace_slot(&mut player, new_slot, selected_slot);
            if res.consumed { to_remove.push(i); }
            to_add.extend(res.drops);
        }

        // Armas → slot seleccionado con estado inicial
        PickupKind::Weapon { wtype, rarity } => {
            if let Some(base) = weapons_catalog.iter().find(|w| w.wtype == wtype) {
                let w = Weapon { rarity, ..*base };
                let ws = WeaponState { ammo_in_mag: w.mag_size, weapon_cd: 0.0, reloading: false, reload_cd: 0.0 };
                let new_slot = SlotItem { item: Item::Weapon(w, ws), count: 1, cd: 0.0 , using: false};
                let res = try_stack_or_replace_slot(&mut player, new_slot, selected_slot);
                if res.consumed { to_remove.push(i); }
                to_add.extend(res.drops);
            }
        }

        // Item genérico
        PickupKind::Item { item, count } => {
            let new_slot = SlotItem { item, count: count.max(1), cd: 0.0 , using: false};
            let res = try_stack_or_replace_slot(&mut player, new_slot, selected_slot);
            if res.consumed { to_remove.push(i); }
            to_add.extend(res.drops);
        }
    }
}

// aplicar cambios fuera del bucle
to_remove.sort_unstable_by(|a,b| b.cmp(a));
for idx in to_remove { pickups.remove(idx); }
pickups.extend(to_add);

        }

        // IA de enemigos
        update_enemies(&mut enemies, &mut player, dt, &ai_cfg, &map, &decorations, &chests, &audio, );

        // Calcula cuántos enemigos siguen vivos
let enemies_left = enemies
    .iter()
    .filter(|e| e.state != EnemyState::Dead && e.hp > 0)
    .count();

// Si no queda ninguno y estamos jugando → pasar a Victory
if enemies_left == 0 && state == GameState::Playing {
    victory.set_stats(elapsed, kills); // tiempo total y kills
    state = GameState::Victory;
    // Saltamos a la rama Victory en el siguiente ciclo (o podrías "continue" aquí)
}


        let attack_down = rl.is_key_down(KeyboardKey::KEY_SPACE);
        // ---------------- DRAW + DISPARO ----------------
let mut d = rl.begin_drawing(&thread);
d.clear_background(Color::BLACK);

let out = draw_frame(
    &mut d,
    &thread,
    &enemies,
    &RenderParams {
        fov,
        proj_dist,
        player_x: player.x,
        player_y: player.y,
        player_angle: player.angle,
    },
    &map,
    &pickups,
    &decorations,
    &chests,
    &wall_textures, // ← NUEVO
     &pickup_tex,
     &chest_tex,
     &enemy_tex,
);

// Disparo/recarga desde el slot seleccionado (arma) — con take()/devolver
if let Some(si) = player.selected {
    if let Some(mut slot) = player.slots[si].take() {
        if let Item::Weapon(w, ref mut ws) = slot.item {
            // Timers
            ws.weapon_cd = (ws.weapon_cd - dt).max(0.0);
            if ws.reloading {
                ws.reload_cd -= dt;
                if ws.reload_cd <= 0.0 {
                    let need = (w.mag_size - ws.ammo_in_mag).max(0);
                    let pool = &mut player.ammo_reserve[w.ammo_type.index()];
                    let take = need.min(*pool);
                    ws.ammo_in_mag += take;
                    *pool -= take;
                    ws.reloading = false;
                    ws.reload_cd = 0.0;
                }
            }

            // Recarga (R)
            // Recarga (R)
if reload_pressed && !ws.reloading {
    let pool = player.ammo_reserve[w.ammo_type.index()];
    if ws.ammo_in_mag < w.mag_size && pool > 0 {
        ws.reloading = true;
        ws.reload_cd = w.effective_reload();
        // 🔊 sonido de recarga
        audio.play_reload(w.wtype);
    }
}


            // Disparo (SPACE)
            if fire_down && !ws.reloading && ws.weapon_cd <= 0.0 {
                if ws.ammo_in_mag > 0 {
                    ws.ammo_in_mag -= 1;

                    // 🔊 SONIDO DE DISPARO — AQUI
                    audio.play_shot(w.wtype);

                    // Hitscan contra sprites de enemigos
                    let center = (SCREEN_W / 2) as i32;
                    let wall_depth = out.zbuffer[center as usize];
                    let mut best: Option<(usize, f32)> = None;

                    for ds in &out.drawn {
                        if center >= ds.start_x
                            && center <= ds.end_x
                            && ds.depth < wall_depth
                        {
                            if best.map_or(true, |(_, bd)| ds.depth < bd) {
                                best = Some((ds.idx, ds.depth));
                            }
                        }
                    }

                    if let Some((hit_idx, _)) = best {
                        if enemies[hit_idx].state != EnemyState::Dead
                            && enemies[hit_idx].hp > 0
                        {
                            // antes de aplicar daño, recordamos si estaba vivo
                            let was_alive = enemies[hit_idx].hp > 0;

                            enemies[hit_idx].hp -= w.effective_damage();
                            if enemies[hit_idx].hp <= 0 {
                                if was_alive {
                                    enemies[hit_idx].state = EnemyState::Dead;
                                    kills += 1; // ✅ contamos el kill sólo al pasar a muerto
                                    audio.play_enemy_death();
                                }
                            } else {
                                enemies[hit_idx].flash_timer = 0.1;
                                 audio.play_enemy_hurt();
                            }
                        }
                    }

                    ws.weapon_cd = w.fire_interval;
                } else {
                    // Auto-recarga si hay reserva
                    let pool = player.ammo_reserve[w.ammo_type.index()];
                    if pool > 0 && !ws.reloading {
                        ws.reloading = true;
                        ws.reload_cd = w.effective_reload();
                        // 🔊 sonido de recarga
                        audio.play_reload(w.wtype);
                    }
                }
            }
        }
        // Devolver el slot
        player.slots[si] = Some(slot);
    }
}


// after: let out = draw_frame(...);

// Botón de ataque (igual que tus armas)


// Si NO hay slot seleccionado (mano vacía = tecla 0) → puñetazo
if attack_down && player.selected.is_none() && player.punch_cd <= 0.0 {
    use crate::consts::{PUNCH_DAMAGE, PUNCH_RANGE, PUNCH_COOLDOWN};
    let center = crate::consts::SCREEN_W / 2;

    // Busca enemigo visible en el centro de pantalla y a rango corto
    let mut best: Option<(usize, f32)> = None; // (idx_enemigo, profundidad)
    for ds in &out.drawn {
        if ds.start_x <= center && center <= ds.end_x && ds.depth <= PUNCH_RANGE {
            // enemigo vivo
            let e = &enemies[ds.idx];
            if e.state != EnemyState::Dead && e.hp > 0 {
                if best.map_or(true, |(_, d)| ds.depth < d) {
                    best = Some((ds.idx, ds.depth));
                }
            }
        }
    }

    if let Some((hit_idx, _)) = best {
        // aplicar daño
        if enemies[hit_idx].hp > 0 {
            enemies[hit_idx].hp -= PUNCH_DAMAGE;
            if enemies[hit_idx].hp <= 0 {
                enemies[hit_idx].state = EnemyState::Dead;
                kills += 1; // ✅ contamos el kill sólo al pasar a muerto
                audio.play_enemy_death();
                // si cuentas kills del jugador aquí:
                // kills += 1;
            } else {
                enemies[hit_idx].flash_timer = 0.1; // feedback visual que ya usas
                audio.play_enemy_hurt();
            }
        }
        player.punch_cd = PUNCH_COOLDOWN;
        // (opcional) feedback de pantalla/sonido
    }
}




        // ---------------- HUD ----------------
        d.draw_text(
            "W/S=adelante/atrás | A/D=girar | ESPACIO=disparar | R=recargar | E=interactuar | 1–5=slot, 0=vacío | F=usar consumible",
            10, 10, 18, Color::WHITE
        );


        d.draw_text(&format!("Munición: "), 10, 58, 18, Color::YELLOW);

        // Reservas de munición
        let names = [AmmoType::Light, AmmoType::Medium, AmmoType::Heavy, AmmoType::Shell, AmmoType::Rocket];
        let mut y = 82;
        for a in names {
            let i = a.index();
            d.draw_text(&format!("{}: {}", a.name(), player.ammo_reserve[i]), 10, y, 16, Color::LIGHTGRAY);
            y += 18;
        }

        // Barra de 5 slots


        // Mira
        let cx = SCREEN_W / 2; let cy = SCREEN_H / 2;
        d.draw_line(cx - 8, cy, cx + 8, cy, Color::WHITE);
        d.draw_line(cx, cy - 8, cx, cy + 8, Color::WHITE);

        minimap.draw(&mut d, &map, player.x, player.y, player.angle);
        // enemigos vivos (ya lo calculabas para otro HUD)
let enemies_left = enemies.iter().filter(|e| e.state != EnemyState::Dead && e.hp > 0).count();

// stats bajo el minimapa
hud::draw_top_right_stats(&mut d, &minimap, elapsed, enemies_left, kills);

hud::draw_bottom_right_health_shield(&mut d, player.hp, player.shield, PLAYER_MAX_HP, PLAYER_MAX_SHIELD);
hud::draw_slots_bar_bottom_left(&mut d, &player.slots, player.selected);

// Después de dibujar tu HUD habitual:
// Cooldown de consumible: abajo al centro (estilo ammo HUD)
if let Some(si) = player.selected {
    if let Some(slot) = &player.slots[si] {
        if let crate::types::Item::Consumable(kind) = slot.item {
            if slot.using && slot.cd > 0.0 {
                use crate::consts::{CD_HEALTH_SMALL, CD_HEALTH_BIG, CD_SHIELD_SMALL, CD_SHIELD_BIG};

                let total = match kind {
                    crate::types::ConsumableType::HealthSmall => CD_HEALTH_SMALL,
                    crate::types::ConsumableType::HealthBig   => CD_HEALTH_BIG,
                    crate::types::ConsumableType::ShieldSmall => CD_SHIELD_SMALL,
                    crate::types::ConsumableType::ShieldBig   => CD_SHIELD_BIG,
                };

                // Nuevo: dibuja centrado abajo
                hud::draw_consumable_cooldown_center_bottom(&mut d, slot.cd, total, kind);
            }
        }
    }
}



// Si el slot seleccionado es arma, mostrar arma empuñada (PNG) + balas (mag/reserva)
if let Some(si) = player.selected {
    if let Some(slot) = &player.slots[si] {
        if let Item::Weapon(w, ws) = &slot.item {
            // 1) Arma empuñada (PNG) centrada abajo, justo encima del HUD de balas
            let tex = weapon_hud_tex.tex(w.wtype);   // ← instancia creada fuera del loop
            hud::draw_held_weapon_center_bottom(&mut d, tex);

            // 2) Contador de balas (mag / reserva)
            let reserve = player.ammo_reserve[w.ammo_type.index()] as i32;
            hud::draw_ammo_center_bottom(&mut d, ws.ammo_in_mag as i32, reserve);
        }
    }
}



    }
}

