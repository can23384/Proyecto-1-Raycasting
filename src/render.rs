use raylib::prelude::*;
use crate::consts::{SCREEN_W, SCREEN_H};
use crate::map::Map;
use crate::types::{
    Enemy, EnemyState, DrawnSprite, Pickup, PickupKind,
    Decoration, DecoKind, Chest,
    Rarity, WeaponType,
};

pub struct RenderParams {
    pub fov: f32,
    pub proj_dist: f32,
    pub player_x: f32,
    pub player_y: f32,
    pub player_angle: f32,
}

pub struct RenderOutput {
    pub zbuffer: Vec<f32>,
    pub drawn: Vec<DrawnSprite>, // rangos visibles de ENEMIGOS para hitscan
}

/// Texturas de pickups (vida/escudo/armas/munición genérica)
pub struct PickupTextures<'a> {
    pub health_small: Option<&'a Texture2D>,
    pub health_big:   Option<&'a Texture2D>,
    pub shield_small: Option<&'a Texture2D>,
    pub shield_big:   Option<&'a Texture2D>,
    // Armas (íconos)
    pub weapon_pistol:  Option<&'a Texture2D>,
    pub weapon_smg:     Option<&'a Texture2D>,
    pub weapon_rifle:   Option<&'a Texture2D>,
    pub weapon_shotgun: Option<&'a Texture2D>,
    pub weapon_rocket:  Option<&'a Texture2D>,
    // Munición (una textura para todos)
    pub ammo_generic:   Option<&'a Texture2D>,
}

/// Texturas de cofres
pub struct ChestTextures<'a> {
    pub closed: Option<&'a Texture2D>,
    pub opened: Option<&'a Texture2D>,
}

/// Texturas de enemigos:
/// - `alive`: único sprite para enemigo vivo
/// - `death_frames`: frames de la animación de muerte (en orden)
/// - `death_frame_time`: duración de cada frame (s)
pub struct EnemyTextures<'a> {
    pub alive: Option<&'a Texture2D>,
    pub death_frames: Vec<&'a Texture2D>,
    pub death_frame_time: f32,
}

/// Visual de un sprite en cola: color plano o textura
#[derive(Clone, Copy)]
enum SpriteVisual<'a> {
    Color(Color),
    Texture(&'a Texture2D),
}

#[derive(Clone, Copy)]
struct QueuedSprite<'a> {
    start_x: i32,
    end_x: i32,
    start_y: i32,
    end_y: i32,
    depth: f32,
    visual: SpriteVisual<'a>,
    glow: Option<Color>,        // resplandor (armas por rareza / flash enemigo)
    enemy_idx: Option<usize>,   // Some(idx) sólo en ENEMIGOS (hitscan)
}

// Color “glow” por rareza (semi-transparente)
fn rarity_glow(r: Rarity) -> Color {
    match r {
        Rarity::Common    => Color::new(180, 180, 180, 110),
        Rarity::Uncommon  => Color::new( 70, 200,  90, 120),
        Rarity::Rare      => Color::new( 80, 140, 255, 120),
        Rarity::Epic      => Color::new(170,  80, 220, 120),
        Rarity::Legendary => Color::new(240, 200,  60, 130),
    }
}

pub fn draw_frame<'a>(
    d: &mut RaylibDrawHandle,
    _thread: &RaylibThread,
    enemies: &[Enemy],
    params: &RenderParams,
    map: &Map,
    pickups: &[Pickup],
    decorations: &[Decoration],
    chests: &[Chest],
    wall_textures: &[Texture2D],         // id de pared 1 → index 0
    pickup_textures: &PickupTextures<'a>,// texturas de pickups
    chest_textures: &ChestTextures<'a>,  // texturas de cofres
    enemy_textures: &EnemyTextures<'a>,  // texturas + animación de enemigos
) -> RenderOutput {
    let mut zbuffer: Vec<f32> = vec![f32::INFINITY; SCREEN_W as usize];
    let mut drawn: Vec<DrawnSprite> = Vec::new();

    let dir_x = params.player_angle.cos();
    let dir_y = params.player_angle.sin();

    // Fondo (cielo/suelo)
    d.draw_rectangle(0, 0, SCREEN_W, SCREEN_H / 2, Color::DARKBLUE);
    d.draw_rectangle(0, SCREEN_H / 2, SCREEN_W, SCREEN_H / 2, Color::DARKBROWN);

    // ----------------- PAREDES: raycasting + texturas por ID -----------------
    for x in 0..SCREEN_W {
        // proyección angular: mapea x de pantalla a [-1..1]
        let camera_x = (2.0 * x as f32 / SCREEN_W as f32) - 1.0;
        let ray_angle = params.player_angle + (camera_x * (params.fov / 2.0));

        let ray_dir_x = ray_angle.cos();
        let ray_dir_y = ray_angle.sin();

        let mut map_x = params.player_x.floor() as i32;
        let mut map_y = params.player_y.floor() as i32;

        let delta_dist_x = if ray_dir_x.abs() < 1e-6 { 1e30 } else { (1.0 / ray_dir_x).abs() };
        let delta_dist_y = if ray_dir_y.abs() < 1e-6 { 1e30 } else { (1.0 / ray_dir_y).abs() };

        let (step_x, mut side_dist_x) = if ray_dir_x < 0.0 {
            (-1, (params.player_x - map_x as f32) * delta_dist_x)
        } else {
            (1, ((map_x as f32 + 1.0) - params.player_x) * delta_dist_x)
        };
        let (step_y, mut side_dist_y) = if ray_dir_y < 0.0 {
            (-1, (params.player_y - map_y as f32) * delta_dist_y)
        } else {
            (1, ((map_y as f32 + 1.0) - params.player_y) * delta_dist_y)
        };

        let mut hit = false;
        let mut side = 0; // 0 = cara X, 1 = cara Y
        let mut steps = 0;

        // DDA
        loop {
            steps += 1;
            if side_dist_x < side_dist_y {
                side_dist_x += delta_dist_x;
                map_x += step_x;
                side = 0;
            } else {
                side_dist_y += delta_dist_y;
                map_y += step_y;
                side = 1;
            }
            // cualquier id > 0 es pared
            if map.at(map_x, map_y) > 0 {
                hit = true;
                break;
            }
            if steps > (map.w + map.h) * 4 { break; } // safety
        }

        if hit {
            // distancia perpendicular (evita fisheye)
            let perp_dist = if side == 0 {
                ((map_x as f32 - params.player_x) + (1 - step_x) as f32 / 2.0) / ray_dir_x
            } else {
                ((map_y as f32 - params.player_y) + (1 - step_y) as f32 / 2.0) / ray_dir_y
            }.abs();

            // Altura proyectada (float)
            let line_h_f = params.proj_dist / perp_dist;

            // Rango "real" sin recortar
            let raw_start = (SCREEN_H as f32 / 2.0) - (line_h_f / 2.0);
            let raw_end   = (SCREEN_H as f32 / 2.0) + (line_h_f / 2.0);

            // Rango visible
            let vis_start = raw_start.max(0.0) as i32;
            let vis_end   = raw_end.min((SCREEN_H - 1) as f32) as i32;

            if vis_end >= vis_start {
                // id de pared y textura
                let tile_id = map.at(map_x, map_y) as usize; // 1..N
                let tex = if tile_id == 0 {
                    &wall_textures[0]
                } else {
                    wall_textures.get(tile_id - 1).unwrap_or(&wall_textures[0])
                };

                // Coordenada “x” en la textura (punto de impacto)
                let mut wall_x = if side == 0 {
                    params.player_y + perp_dist * ray_dir_y
                } else {
                    params.player_x + perp_dist * ray_dir_x
                };
                wall_x -= wall_x.floor();

                let tex_w = tex.width;
                let tex_h = tex.height;
                let mut tex_x = (wall_x * tex_w as f32) as i32;

                // Corregir espejo según cara
                if side == 0 && ray_dir_x > 0.0 { tex_x = tex_w - tex_x - 1; }
                if side == 1 && ray_dir_y < 0.0 { tex_x = tex_w - tex_x - 1; }

                // Mapeo vertical correcto al clipear
                let tex_step = tex_h as f32 / line_h_f;
                let mut tex_y_start = 0.0_f32;
                if raw_start < 0.0 {
                    tex_y_start = -raw_start * tex_step;
                }
                let visible_px = (vis_end - vis_start + 1) as f32;
                let src_h = visible_px * tex_step;

                let src = Rectangle { x: tex_x as f32, y: tex_y_start, width: 1.0, height: src_h };
                let dest= Rectangle { x: x as f32,    y: vis_start as f32, width: 1.0, height: visible_px };

                // Sombrear caras Y para profundidad
                let tint = if side == 1 { Color::GRAY } else { Color::WHITE };
                d.draw_texture_pro(tex, src, dest, Vector2::new(0.0, 0.0), 0.0, tint);

                // zbuffer por columna
                zbuffer[x as usize] = perp_dist;
            }
        }
    }

    // ----------------- SPRITES: enemigos + pickups + deco + cofres -----------------
    // Plano de cámara para proyección de sprites
    let tan_half = (params.fov * 0.5).tan();
    let plane_x = -dir_y * tan_half;
    let plane_y =  dir_x * tan_half;

    let mut sprites: Vec<QueuedSprite> = Vec::new();

// ---- Enemigos (textura de vivo o frame de muerte) ----
for (idx, e) in enemies.iter().enumerate() {
    // proyectar (incluye muertos para mostrar anim y cadáver)
    let rel_x = e.x - params.player_x;
    let rel_y = e.y - params.player_y;

    let inv_det = 1.0 / (plane_x * dir_y - dir_x * plane_y);
    let trans_x = inv_det * (dir_y * rel_x - dir_x * rel_y);
    let trans_y = inv_det * (-plane_y * rel_x + plane_x * rel_y);
    if trans_y <= 0.0001 { continue; }

    let sprite_screen_x = (SCREEN_W as f32 / 2.0) * (1.0 + trans_x / trans_y);
    let sprite_h = (params.proj_dist / trans_y) as i32;
    let sprite_w = sprite_h;

    let start_y = (SCREEN_H / 2 - sprite_h / 2).max(0);
    let end_y   = (SCREEN_H / 2 + sprite_h / 2).min(SCREEN_H - 1);
    let start_x = ((sprite_screen_x as i32) - sprite_w / 2).max(0);
    let end_x   = ((sprite_screen_x as i32) + sprite_w / 2).min(SCREEN_W - 1);

    if end_x >= start_x && end_y > start_y {
        // Elegimos visual según estado
        let (vis, glow) = match e.state {
            EnemyState::Dead => {
                // Animación de muerte por frames
                let frames = &enemy_textures.death_frames;
                if !frames.is_empty() {
                    let ft = enemy_textures.death_frame_time.max(0.0001);
                    let idxf = ((e.death_anim_t / ft).floor() as usize)
                        .min(frames.len() - 1);
                    (SpriteVisual::Texture(frames[idxf]), None)
                } else {
                    (SpriteVisual::Color(Color::DARKGRAY), None)
                }
            }
            _ => {
                let vis = enemy_textures.alive
                    .map(SpriteVisual::Texture)
                    .unwrap_or(SpriteVisual::Color(if e.flash_timer > 0.0 { Color::YELLOW } else { e.color }));
                let glow = if e.flash_timer > 0.0 { Some(Color::new(255, 255, 0, 140)) } else { None };
                (vis, glow)
            }
        };

        sprites.push(QueuedSprite {
            start_x, end_x, start_y, end_y, depth: trans_y,
            visual: vis,
            glow,
            // Para hitscan: reporta SOLO vivos. Si quieres que el cadáver no sea target:
            enemy_idx: if matches!(e.state, EnemyState::Dead) { None } else { Some(idx) },
        });
    }
}

    // ---- Pickups (texturizados si hay; si no, color) ----
    const PK_NEAR: f32 = 0.2;
    const PK_SCALE: f32 = 0.35;
    const PK_MAX_FRAC: f32 = 0.18;

    for p in pickups {
        let rel_x = p.x - params.player_x;
        let rel_y = p.y - params.player_y;

        let inv_det = 1.0 / (plane_x * dir_y - dir_x * plane_y);
        let trans_x = inv_det * (dir_y * rel_x - dir_x * rel_y);
        let trans_y = inv_det * (-plane_y * rel_x + plane_x * rel_y);
        if trans_y <= 0.0001 || trans_y < PK_NEAR { continue; }

        let sprite_screen_x = (SCREEN_W as f32 / 2.0) * (1.0 + trans_x / trans_y);
        let phys_h = (params.proj_dist / trans_y) as i32;

        let mut sprite_h = (phys_h as f32 * PK_SCALE) as i32;
        let mut sprite_w = sprite_h;
        let max_h = (SCREEN_H as f32 * PK_MAX_FRAC) as i32;
        sprite_h = sprite_h.clamp(2, max_h);
        sprite_w = sprite_w.clamp(2, max_h);

        let base_y = (SCREEN_H / 2 + phys_h / 2).min(SCREEN_H - 1);
        let start_y = (base_y - sprite_h).max(0);
        let end_y   = base_y;

        let start_x = ((sprite_screen_x as i32) - sprite_w / 2).max(0);
        let end_x   = ((sprite_screen_x as i32) + sprite_w / 2).min(SCREEN_W - 1);

        if end_x >= start_x && end_y > start_y {
            // Selección de visual + glow (para armas)
            let (visual, glow) = match p.kind {
                PickupKind::HealthSmall => (
                    pickup_textures.health_small.map(SpriteVisual::Texture)
                        .unwrap_or(SpriteVisual::Color(p.color)),
                    None
                ),
                PickupKind::HealthBig => (
                    pickup_textures.health_big.map(SpriteVisual::Texture)
                        .unwrap_or(SpriteVisual::Color(p.color)),
                    None
                ),
                PickupKind::ShieldSmall => (
                    pickup_textures.shield_small.map(SpriteVisual::Texture)
                        .unwrap_or(SpriteVisual::Color(p.color)),
                    None
                ),
                PickupKind::ShieldBig => (
                    pickup_textures.shield_big.map(SpriteVisual::Texture)
                        .unwrap_or(SpriteVisual::Color(p.color)),
                    None
                ),
                // Armas: textura por tipo + glow por rareza
                PickupKind::Weapon { wtype, rarity } => {
                    let tex_opt = match wtype {
                        WeaponType::Pistol         => pickup_textures.weapon_pistol,
                        WeaponType::SMG            => pickup_textures.weapon_smg,
                        WeaponType::Rifle          => pickup_textures.weapon_rifle,
                        WeaponType::Shotgun        => pickup_textures.weapon_shotgun,
                        WeaponType::RocketLauncher => pickup_textures.weapon_rocket,
                    };
                    let vis = tex_opt
                        .map(SpriteVisual::Texture)
                        .unwrap_or(SpriteVisual::Color(p.color));
                    (vis, Some(rarity_glow(rarity)))
                }
                // Munición → textura genérica (si hay)
                PickupKind::Ammo { .. } => (
                    pickup_textures.ammo_generic
                        .map(SpriteVisual::Texture)
                        .unwrap_or(SpriteVisual::Color(p.color)),
                    None
                ),
                // otros (Item genérico…)
                _ => (SpriteVisual::Color(p.color), None),
            };

            sprites.push(QueuedSprite {
                start_x, end_x, start_y, end_y, depth: trans_y,
                visual,
                glow,
                enemy_idx: None,
            });
        }
    }

    // ---- Decoraciones ----
    const DECO_SCALE_BLOCK: f32 = 0.70;
    const DECO_SCALE_GHOST: f32 = 0.55;

    for deco in decorations {
        let rel_x = deco.x - params.player_x;
        let rel_y = deco.y - params.player_y;

        let inv_det = 1.0 / (plane_x * dir_y - dir_x * plane_y);
        let trans_x = inv_det * (dir_y * rel_x - dir_x * rel_y);
        let trans_y = inv_det * (-plane_y * rel_x + plane_x * rel_y);
        if trans_y <= 0.0001 { continue; }

        let sprite_screen_x = (SCREEN_W as f32 / 2.0) * (1.0 + trans_x / trans_y);
        let phys_h = (params.proj_dist / trans_y) as i32;

        let scale = match deco.kind {
            DecoKind::Blocking => DECO_SCALE_BLOCK,
            DecoKind::Ghost    => DECO_SCALE_GHOST,
        };

        let mut sprite_h = (phys_h as f32 * scale) as i32;
        let mut sprite_w = sprite_h;
        let max_h = (SCREEN_H as f32 * 0.35) as i32;
        sprite_h = sprite_h.clamp(2, max_h);
        sprite_w = sprite_w.clamp(2, max_h);

        let base_y = (SCREEN_H / 2 + phys_h / 2).min(SCREEN_H - 1);
        let start_y = (base_y - sprite_h).max(0);
        let end_y   = base_y;

        let start_x = ((sprite_screen_x as i32) - sprite_w / 2).max(0);
        let end_x   = ((sprite_screen_x as i32) + sprite_w / 2).min(SCREEN_W - 1);

        let color = match deco.kind {
            DecoKind::Blocking => Color::BROWN,
            DecoKind::Ghost    => Color::LIGHTGRAY,
        };

        if end_x >= start_x && end_y > start_y {
            sprites.push(QueuedSprite {
                start_x, end_x, start_y, end_y, depth: trans_y,
                visual: SpriteVisual::Color(color),
                glow: None,
                enemy_idx: None,
            });
        }
    }

    // ---- Cofres ----
    for c in chests {
        let rel_x = c.x - params.player_x;
        let rel_y = c.y - params.player_y;

        let inv_det = 1.0 / (plane_x * dir_y - dir_x * plane_y);
        let trans_x = inv_det * (dir_y * rel_x - dir_x * rel_y);
        let trans_y = inv_det * (-plane_y * rel_x + plane_x * rel_y);
        if trans_y <= 0.0001 { continue; }

        let sprite_screen_x = (SCREEN_W as f32 / 2.0) * (1.0 + trans_x / trans_y);
        let phys_h = (params.proj_dist / trans_y) as i32;

        let mut sprite_h = (phys_h as f32 * 0.55) as i32;
        let mut sprite_w = sprite_h;
        let max_h = (SCREEN_H as f32 * 0.30) as i32;
        sprite_h = sprite_h.clamp(2, max_h);
        sprite_w = sprite_w.clamp(2, max_h);

        let base_y = (SCREEN_H / 2 + phys_h / 2).min(SCREEN_H - 1);
        let start_y = (base_y - sprite_h).max(0);
        let end_y   = base_y;

        let start_x = ((sprite_screen_x as i32) - sprite_w / 2).max(0);
        let end_x   = ((sprite_screen_x as i32) + sprite_w / 2).min(SCREEN_W - 1);

        // textura por estado
        let tex_opt = if c.opened {
            chest_textures.opened
        } else {
            chest_textures.closed
        };

        let fallback_col = if c.opened { Color::YELLOW } else { Color::GOLD };

        if end_x >= start_x && end_y > start_y {
            let visual = tex_opt
                .map(SpriteVisual::Texture)
                .unwrap_or(SpriteVisual::Color(fallback_col));

            sprites.push(QueuedSprite {
                start_x, end_x, start_y, end_y, depth: trans_y,
                visual,
                glow: None,
                enemy_idx: None,
            });
        }
    }

    // Orden de pintado: lejos → cerca (para zbuffer correcto)
    sprites.sort_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap_or(std::cmp::Ordering::Equal));

    // Dibujar sprites con zbuffer
    for s in &sprites {
        match s.visual {
            SpriteVisual::Color(col) => {
                for stripe in s.start_x..=s.end_x {
                    let x = stripe as usize;
                    if x < zbuffer.len() && s.depth < zbuffer[x] {
                        d.draw_line(stripe, s.start_y, stripe, s.end_y, col);
                    }
                }
            }
            SpriteVisual::Texture(tex) => {
                let tex_w = tex.width as f32;
                let tex_h = tex.height as f32;
                let sprite_w = (s.end_x - s.start_x + 1) as f32;
                let visible_h = (s.end_y - s.start_y + 1) as f32;

                for stripe in s.start_x..=s.end_x {
                    let x = stripe as usize;
                    if x >= zbuffer.len() || !(s.depth < zbuffer[x]) { continue; }

                    // 🔆 GLOW (debajo de la textura), ligeramente más alto
                    if let Some(glow) = s.glow {
                        let gy1 = (s.start_y - 2).max(0);
                        let gy2 = (s.end_y + 2).min(SCREEN_H - 1);
                        d.draw_line(stripe, gy1, stripe, gy2, glow);
                    }

                    // u en [0..1] para esta columna
                    let u = (stripe - s.start_x) as f32 / sprite_w;
                    let tex_x = (u * (tex_w - 1.0)).clamp(0.0, tex_w - 1.0);

                    let src = Rectangle { x: tex_x, y: 0.0, width: 1.0, height: tex_h };
                    let dest= Rectangle {
                        x: stripe as f32,
                        y: s.start_y as f32,
                        width: 1.0,
                        height: visible_h,
                    };

                    d.draw_texture_pro(tex, src, dest, Vector2::new(0.0, 0.0), 0.0, Color::WHITE);
                }
            }
        }
    }

    // Rangos de enemigos para hitscan
    for s in &sprites {
        if let Some(idx) = s.enemy_idx {
            drawn.push(DrawnSprite {
                start_x: s.start_x,
                end_x: s.end_x,
                depth: s.depth,
                idx,
            });
        }
    }

    RenderOutput { zbuffer, drawn }
}
