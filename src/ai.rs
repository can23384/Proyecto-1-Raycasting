use std::cmp::Ordering;
use raylib::prelude::Color;

use crate::map::Map;
use crate::types::{Enemy, EnemyState, Player, Decoration, Chest};
use crate::audio::Audio; 

/// Línea de visión: DDA sobre la rejilla de paredes.
/// Devuelve true si no hay una pared entre (sx,sy) y (tx,ty).
pub fn has_los(map: &Map, sx: f32, sy: f32, tx: f32, ty: f32) -> bool {
    let dx = tx - sx;
    let dy = ty - sy;
    let dist = (dx * dx + dy * dy).sqrt().max(1e-6);
    let dir_x = dx / dist;
    let dir_y = dy / dist;

    let mut map_x = sx.floor() as i32;
    let mut map_y = sy.floor() as i32;

    let delta_dist_x = if dir_x.abs() < 1e-6 { f32::INFINITY } else { (1.0 / dir_x).abs() };
    let delta_dist_y = if dir_y.abs() < 1e-6 { f32::INFINITY } else { (1.0 / dir_y).abs() };

    let (step_x, mut side_dist_x) = if dir_x < 0.0 {
        (-1, (sx - map_x as f32) * delta_dist_x)
    } else {
        ( 1, ((map_x as f32 + 1.0) - sx) * delta_dist_x)
    };
    let (step_y, mut side_dist_y) = if dir_y < 0.0 {
        (-1, (sy - map_y as f32) * delta_dist_y)
    } else {
        ( 1, ((map_y as f32 + 1.0) - sy) * delta_dist_y)
    };

    let target_cell_x = tx.floor() as i32;
    let target_cell_y = ty.floor() as i32;

    // Límite de pasos razonable
    let max_steps = (map.w + map.h) * 4;
    for _ in 0..max_steps {
        if side_dist_x < side_dist_y {
            side_dist_x += delta_dist_x;
            map_x += step_x;
        } else {
            side_dist_y += delta_dist_y;
            map_y += step_y;
        }

        // ¿Llegamos a la celda del objetivo?
        if map_x == target_cell_x && map_y == target_cell_y {
            return true;
        }

        // ¿Pared bloqueando?
        if map.at(map_x, map_y) == 1 {
            return false;
        }
    }
    false
}

/// Aplica daño al jugador: primero escudo, luego vida.
/// Devuelve (hubo_daño, murió_en_esta_llamada)
fn apply_damage(player: &mut Player, mut dmg: i32) -> (bool, bool) {
    if dmg <= 0 { return (false, false); }

    let hp_before = player.hp;
    let sh_before = player.shield;
    let was_alive = hp_before > 0;

    // absorción por escudo
    let absorbed = player.shield.min(dmg);
    player.shield -= absorbed;
    dmg -= absorbed;

    // daño restante a vida
    if dmg > 0 {
        player.hp -= dmg;
        if player.hp < 0 { player.hp = 0; }
    }

    let took = (player.hp != hp_before) || (player.shield != sh_before);
    let died_now = was_alive && player.hp <= 0;
    (took, died_now)
}


pub struct AiCfg {
    pub detect_radius: f32, // distancia a la que pasa de Idle a Chase si hay LOS
    pub melee_range: f32,   // rango de melee
    pub melee_dps: f32,     // daño por segundo de melee
    pub shoot_range: f32,   // rango máximo de disparo
}

/// Actualiza todos los enemigos: movimiento, colisiones, melee y disparo.
/// - Respeta colisiones contra paredes y decoraciones bloqueantes.
/// - Disparo usa daño/cadencia del arma de cada enemigo.
/// - Cambia color para feedback (amarillo al disparar/recibir daño, rosa herido, gris muerto).
pub fn update_enemies(
    enemies: &mut [Enemy],
    player: &mut Player,
    dt: f32,
    cfg: &AiCfg,
    map: &Map,
    decorations: &[Decoration],
    chests: &[Chest],
    audio: &Audio, 
) {
    // Orden opcional (útil para render si reutilizas el orden): más lejos primero.
    enemies.sort_by(|a, b| {
        let da = (a.x - player.x).powi(2) + (a.y - player.y).powi(2);
        let db = (b.x - player.x).powi(2) + (b.y - player.y).powi(2);
        db.partial_cmp(&da).unwrap_or(Ordering::Equal)
    });

    for e in enemies.iter_mut() {
        // Timers
        e.weapon_cd = (e.weapon_cd - dt).max(0.0);
        e.flash_timer = (e.flash_timer - dt).max(0.0);

        // Normalizar estado por vida
        if e.hp <= 0 {
    if e.state != EnemyState::Dead {
        e.state = EnemyState::Dead;
        e.death_anim_t = 0.0; // reiniciar anim
    }
}

// Si está muerto: avanzar animación y NO hacer IA
if e.state == EnemyState::Dead {
    e.death_anim_t += dt;   // 👈 sin esto no avanza de frame
    continue;
}

        // Vector hacia el jugador
        let dx = player.x - e.x;
        let dy = player.y - e.y;
        let dist = (dx * dx + dy * dy).sqrt();

        // Cambiar a persecución si está cerca y con LOS
        if e.state == EnemyState::Idle && dist < cfg.detect_radius && has_los(map, e.x, e.y, player.x, player.y) {
            e.state = EnemyState::Chase;
        }

        if e.state == EnemyState::Chase {
            // Dirección normalizada (si dist=0, no mueve)
            let (dir_x, dir_y) = if dist > 1e-4 {
                (dx / dist, dy / dist)
            } else {
                (0.0, 0.0)
            };

            // Movimiento propuesto
            let mut mvx = dir_x * e.speed * dt;
            let mut mvy = dir_y * e.speed * dt;

            // Colisión simple (AABB por ejes y radio)
            let radius = 0.2_f32;

            // Helper: colisión contra decoraciones bloqueantes
            let collides_deco = |x: f32, y: f32| -> bool {
                decorations.iter().any(|d| {
                    if !d.is_blocking() { return false; }
                    let dx = d.x - x;
                    let dy = d.y - y;
                    let rr = radius + d.radius;
                    (dx * dx + dy * dy) < (rr * rr)
                })
            };

            // Avance eje X
            let next_x = e.x + mvx;
            let can_x = map.at((next_x - radius).floor() as i32, e.y.floor() as i32) == 0
                && map.at((next_x + radius).floor() as i32, e.y.floor() as i32) == 0
                && !collides_deco(next_x, e.y);
            if can_x {
                e.x = next_x;
            } else {
                mvx = 0.0;
            }

            // Avance eje Y
            let next_y = e.y + mvy;
            let can_y = map.at(e.x.floor() as i32, (next_y - radius).floor() as i32) == 0
                && map.at(e.x.floor() as i32, (next_y + radius).floor() as i32) == 0
                && !collides_deco(e.x, next_y);
            if can_y {
                e.y = next_y;
            } else {
                mvy = 0.0;
            }

            // Recalcular distancia tras mover
            let ndx = player.x - e.x;
            let ndy = player.y - e.y;
            let ndist = (ndx * ndy + ndy * ndy).sqrt(); // <-- ojo: esto tiene un bug (dx*dy). Corrígelo abajo.

            // CORRECCIÓN del cálculo de ndist (arriba hay un typo). Usa esto:
            let ndist = {
                let dx2 = player.x - e.x;
                let dy2 = player.y - e.y;
                (dx2 * dx2 + dy2 * dy2).sqrt()
            };

            // Melee si está muy cerca
if ndist < cfg.melee_range {
    let (took, died_now) = apply_damage(player, (cfg.melee_dps * dt) as i32);
    if died_now {
        audio.play_player_death();
    } else if took {
        audio.play_player_hurt();
    }
}

// Disparo si está dentro de rango y con LOS y cooldown listo
if ndist <= cfg.shoot_range && e.weapon_cd <= 0.0 && has_los(map, e.x, e.y, player.x, player.y) {
    let (took, died_now) = apply_damage(player, e.weapon.damage);
    e.weapon_cd = e.weapon.fire_interval;
    e.flash_timer = 0.08;

    if died_now {
        audio.play_player_death();
    } else if took {
        audio.play_player_hurt();
    }
}

        }

        // Feedback visual según estado/vida
        e.color = if e.hp <= 0 {
            Color::DARKGRAY
        } else if e.flash_timer > 0.0 {
            Color::YELLOW
        } else if e.hp > 40 {
            Color::ORANGE
        } else {
            Color::PINK
        };
    }
}
