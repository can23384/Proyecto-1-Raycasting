use std::fs;
use std::path::Path;

pub struct Map {
    pub w: usize,
    pub h: usize,
    data: Vec<u8>,
    pub player_spawn: Option<(f32, f32)>,
    pub enemy_spawns: Vec<(f32, f32)>,

    // (ya existentes) salud/escudo:
    pub health_small_spawns: Vec<(f32, f32)>,
    pub health_big_spawns:   Vec<(f32, f32)>,
    pub shield_small_spawns: Vec<(f32, f32)>,
    pub shield_big_spawns:   Vec<(f32, f32)>,

    // (ya existentes) decoraciones:
    pub deco_block_spawns: Vec<(f32, f32)>,
    pub deco_ghost_spawns: Vec<(f32, f32)>,

    // (ya existente) cofres:
    pub chest_spawns: Vec<(f32, f32)>,

    // (ya existentes) munición:
    pub ammo_light_spawns:  Vec<(f32, f32)>,
    pub ammo_medium_spawns: Vec<(f32, f32)>,
    pub ammo_heavy_spawns:  Vec<(f32, f32)>,
    pub ammo_shell_spawns:  Vec<(f32, f32)>,
    pub ammo_rocket_spawns: Vec<(f32, f32)>,

    // 🔹 NUEVO: spawns de armas
    pub weapon_pistol_spawns: Vec<(f32, f32)>, // A
    pub weapon_smg_spawns:    Vec<(f32, f32)>, // M
    pub weapon_rifle_spawns:  Vec<(f32, f32)>, // R
    pub weapon_shotgun_spawns:Vec<(f32, f32)>, // O
    pub weapon_rocket_spawns: Vec<(f32, f32)>, // K

    pub heal_random_spawns: Vec<(f32, f32)>,   // lugares donde puede salir vida pequeña / vida mayor / nada
    pub shield_random_spawns: Vec<(f32, f32)>, // lugares donde puede salir escudo menor / escudo mayor / nada
    pub weapon_random_spawns: Vec<(f32, f32)>, // puntos donde puede aparecer un arma aleatoria o nada

    pub ammo_random_spawns: Vec<(f32, f32)>,

}



impl Map {
    pub fn from_txt<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let txt = fs::read_to_string(&path).map_err(|e| format!("No se pudo leer el mapa: {e}"))?;
        let lines: Vec<&str> = txt.lines().filter(|l| !l.trim().is_empty()).collect();
        if lines.is_empty() { return Err("El archivo de mapa está vacío".into()); }

        let w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let h = lines.len();

        let mut data = Vec::with_capacity(w * h);
        let mut player_spawn: Option<(f32, f32)> = None;
        let mut enemy_spawns = Vec::new();

        let mut health_small_spawns = Vec::new();
        let mut health_big_spawns   = Vec::new();
        let mut shield_small_spawns = Vec::new();
        let mut shield_big_spawns   = Vec::new();
        let mut deco_block_spawns = Vec::new();
        let mut deco_ghost_spawns = Vec::new();
        let mut chest_spawns = Vec::new();

        let mut ammo_light_spawns  = Vec::new();
        let mut ammo_medium_spawns = Vec::new();
        let mut ammo_heavy_spawns  = Vec::new();
        let mut ammo_shell_spawns  = Vec::new();
        let mut ammo_rocket_spawns = Vec::new();

        let mut weapon_pistol_spawns = Vec::new();
        let mut weapon_smg_spawns    = Vec::new();
        let mut weapon_rifle_spawns  = Vec::new();
        let mut weapon_shotgun_spawns= Vec::new();
        let mut weapon_rocket_spawns = Vec::new();

        let mut heal_random_spawns = Vec::new();
        let mut shield_random_spawns = Vec::new();

        let mut weapon_random_spawns = Vec::new();

        let mut ammo_random_spawns = Vec::new();

        for (y, raw) in lines.iter().enumerate() {
            let mut row: Vec<char> = raw.chars().collect();
            if row.len() < w { row.resize(w, '1'); }
            for (x, ch) in row.into_iter().enumerate() {
                match ch {
                        // PAREDES
    '1'..='9' => {
        let id = (ch as u8 - b'0') as u8; // 1..9
        data.push(id);
    }
    '#' => { data.push(1); } // compat: muro básico

    // SUELO
    '0' | ' ' | '.' => data.push(0),

    // Player / Enemigos
    'P' => { data.push(0); if player_spawn.is_none() { player_spawn = Some((x as f32 + 0.5, y as f32 + 0.5)); } }
    'E' => { data.push(0); enemy_spawns.push((x as f32 + 0.5, y as f32 + 0.5)); }

                    // 🔹 nuevos pickups:
                    'H' => { data.push(0); health_big_spawns.push((x as f32 + 0.5, y as f32 + 0.5)); }
                    'S' => { data.push(0); shield_big_spawns.push((x as f32 + 0.5, y as f32 + 0.5)); }
                    'B' => { data.push(0); deco_block_spawns.push((x as f32 + 0.5, y as f32 + 0.5)); }
                    'b' => { data.push(0); deco_ghost_spawns.push((x as f32 + 0.5, y as f32 + 0.5)); }
                    'C' => { data.push(0); chest_spawns.push((x as f32 + 0.5, y as f32 + 0.5)); }

                    // 🔹 munición
                    't' => { data.push(0); ammo_light_spawns.push((x as f32 + 0.5, y as f32 + 0.5)); }
                    'y' => { data.push(0); ammo_medium_spawns.push((x as f32 + 0.5, y as f32 + 0.5)); }
                    'u' => { data.push(0); ammo_heavy_spawns.push((x as f32 + 0.5, y as f32 + 0.5)); }
                    'g' => { data.push(0); ammo_shell_spawns.push((x as f32 + 0.5, y as f32 + 0.5)); }
                    'r' => { data.push(0); ammo_rocket_spawns.push((x as f32 + 0.5, y as f32 + 0.5)); }


                    'A' => { data.push(0); weapon_pistol_spawns.push((x as f32 + 0.5, y as f32 + 0.5)); }
                    'M' => { data.push(0); weapon_smg_spawns.push(   (x as f32 + 0.5, y as f32 + 0.5)); }
                    'R' => { data.push(0); weapon_rifle_spawns.push( (x as f32 + 0.5, y as f32 + 0.5)); }
                    'O' => { data.push(0); weapon_shotgun_spawns.push((x as f32 + 0.5, y as f32 + 0.5)); }
                    'K' => { data.push(0); weapon_rocket_spawns.push((x as f32 + 0.5, y as f32 + 0.5)); }

                    'h' => {
                        // spawn aleatorio de vida
                        data.push(0); // ← suelo (antes intentabas usar cells[idx])
                        heal_random_spawns.push((x as f32 + 0.5, y as f32 + 0.5));
                    }
                    's' => {
                        // spawn aleatorio de escudo
                        data.push(0); // ← suelo
                        shield_random_spawns.push((x as f32 + 0.5, y as f32 + 0.5));
                    }

                    'w' => {
                        data.push(0); // suelo
                        weapon_random_spawns.push((x as f32 + 0.5, y as f32 + 0.5));
                    }

                    'm' => {
    data.push(0);
    ammo_random_spawns.push((x as f32 + 0.5, y as f32 + 0.5));
}



                    _ => data.push(0),
                }
            }
        }

        Ok(Self {
            w, h, data,
            player_spawn, enemy_spawns,
            health_small_spawns, health_big_spawns, shield_small_spawns, shield_big_spawns,
            deco_block_spawns, deco_ghost_spawns,
            chest_spawns,
            ammo_light_spawns, ammo_medium_spawns, ammo_heavy_spawns, ammo_shell_spawns, ammo_rocket_spawns,
            weapon_pistol_spawns,
    weapon_smg_spawns,
    weapon_rifle_spawns,
    weapon_shotgun_spawns,
    weapon_rocket_spawns,
    heal_random_spawns,
    shield_random_spawns,
    weapon_random_spawns, ammo_random_spawns,
        })
    }

    #[inline]
    pub fn at(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h { 1 }
        else { self.data[y as usize * self.w + x as usize] }
    }


    pub fn is_solid(&self, x: i32, y: i32) -> bool {
    self.at(x, y) > 0 // cualquier id > 0 es pared
}
}
