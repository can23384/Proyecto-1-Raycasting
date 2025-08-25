use raylib::prelude::*;
use std::f32::consts::PI;
use crate::map::Map;
use crate::types::{Player, Decoration, Chest};

pub struct MoveCfg {
    pub move_speed: f32,
    pub rot_speed: f32,
}

pub fn handle_input(
    rl: &RaylibHandle,
    player: &mut Player,
    cfg: &MoveCfg,
    dt: f32,
    map: &Map,
    decorations: &[Decoration],
    chests: &[Chest], // <-- nuevo
) {
    if rl.is_key_down(KeyboardKey::KEY_A) { player.angle -= cfg.rot_speed * dt; }
    if rl.is_key_down(KeyboardKey::KEY_D) { player.angle += cfg.rot_speed * dt; }

    if player.angle > PI { player.angle -= 2.0 * PI; }
    if player.angle < -PI { player.angle += 2.0 * PI; }

    let dir_x = player.angle.cos();
    let dir_y = player.angle.sin();

    let mut next_px = player.x;
    let mut next_py = player.y;

    let radius = 0.2_f32;

    // helper de colisión con decoraciones bloqueantes
    let collides_deco = |x: f32, y: f32| -> bool {
        decorations.iter().any(|d| {
            if !d.is_blocking() { return false; }
            let dx = d.x - x;
            let dy = d.y - y;
            (dx*dx + dy*dy).sqrt() < (radius + d.radius)
        })
    };

    let collides_chest = |x: f32, y: f32| -> bool {
    chests.iter().any(|c| {
        if !c.is_blocking() { return false; }
        let dx = c.x - x;
        let dy = c.y - y;
        (dx*dx + dy*dy) < ( (radius + c.radius)*(radius + c.radius) )
    })
};

    // eje X
    if map.at(next_px.floor() as i32, player.y.floor() as i32) == 0 && !collides_deco(next_px, player.y) {
        player.x = next_px;
    }
    // eje Y
    if map.at(player.x.floor() as i32, next_py.floor() as i32) == 0 && !collides_deco(player.x, next_py) {
        player.y = next_py;
    }

    if rl.is_key_down(KeyboardKey::KEY_W) {
        next_px += dir_x * cfg.move_speed * dt;
        next_py += dir_y * cfg.move_speed * dt;
    }
    if rl.is_key_down(KeyboardKey::KEY_S) {
        next_px -= dir_x * cfg.move_speed * dt;
        next_py -= dir_y * cfg.move_speed * dt;
    }

    if map.at(next_px.floor() as i32, player.y.floor() as i32) == 0 { player.x = next_px; }
    if map.at(player.x.floor() as i32, next_py.floor() as i32) == 0 { player.y = next_py; }
}
