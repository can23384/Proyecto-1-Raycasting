use raylib::prelude::*;
use crate::consts::{SCREEN_W, SCREEN_H};
use crate::map::Map;
use crate::types::{SlotItem, Item, ConsumableType, Rarity, Weapon};

pub struct Minimap {
    /// ¿está expandido?
    pub expanded: bool,
    /// tamaño (ancho, alto) en píxeles cuando está contraído
    small_size: (i32, i32),
    /// tamaño (ancho, alto) cuando está expandido
    large_size: (i32, i32),
    /// margen desde los bordes de la pantalla
    margin: i32,
}

impl Default for Minimap {
    fn default() -> Self {
        Self {
            expanded: false,
            small_size: (180, 140),
            large_size: (420, 320),
            margin: 10,
        }
    }
}

impl Minimap {
    pub fn new() -> Self { Self::default() }

    /// Llama cada frame para manejar la tecla M
    pub fn handle_input(&mut self, rl: &RaylibHandle) {
        if rl.is_key_pressed(KeyboardKey::KEY_M) {
            self.expanded = !self.expanded;
        }
    }

    /// Devuelve (x, y, w, h) del rectángulo del minimapa según su estado actual.
    pub fn bounds(&self) -> (i32, i32, i32, i32) {
        let (mw, mh) = if self.expanded { self.large_size } else { self.small_size };
        let x0 = SCREEN_W - self.margin - mw;
        let y0 = self.margin;
        (x0, y0, mw, mh)
    }

    /// Dibuja el minimapa (paredes + jugador con flecha de orientación)
    pub fn draw(&self, d: &mut RaylibDrawHandle, map: &Map, player_x: f32, player_y: f32, player_angle: f32) {
        // tamaño actual según estado
        let (mw, mh) = if self.expanded { self.large_size } else { self.small_size };

        let x0 = SCREEN_W - self.margin - mw;
        let y0 = self.margin;

        // fondo semitransparente + marco
        d.draw_rectangle(x0 - 2, y0 - 2, mw + 4, mh + 4, Color::GRAY);
        d.draw_rectangle(x0, y0, mw, mh, Color::new(0, 0, 0, 180));

        // calcular tamaño de celda
        let map_w = map.w as i32;
        let map_h = map.h as i32;
        if map_w == 0 || map_h == 0 { return; }

        let sx = mw as f32 / map_w as f32;
        let sy = mh as f32 / map_h as f32;
        let tile = sx.min(sy); // mantener proporción y caber dentro

        // offset para centrar si sobra espacio por proporcionalidad
        let used_w = (tile * map_w as f32) as i32;
        let used_h = (tile * map_h as f32) as i32;
        let ox = x0 + (mw - used_w) / 2;
        let oy = y0 + (mh - used_h) / 2;

        // dibujar paredes
        let wall_col = Color::LIGHTGRAY;
        let tile_i = tile.max(1.0) as i32;
        for my in 0..map_h {
            for mx in 0..map_w {
                if map.at(mx as i32, my as i32) == 1 {
                    let rx = ox + (mx as f32 * tile) as i32;
                    let ry = oy + (my as f32 * tile) as i32;
                    d.draw_rectangle(rx, ry, tile_i, tile_i, wall_col);
                }
            }
        }

        // ------------------ Jugador: flecha de orientación ------------------
        // Centro del jugador en el minimapa (en píxeles)
        let px = ox as f32 + player_x * tile;
        let py = oy as f32 + player_y * tile;

        // Tamaño relativo de la flecha según el tile/celda
        let head_len   = (tile * 0.8).clamp(6.0, 18.0);  // largo punta
        let base_half  = (tile * 0.35).clamp(3.0, 10.0); // media base
        let back_len   = (tile * 0.35).clamp(3.0, 10.0); // hacia atrás desde el centro

        // Dirección hacia delante
        let dirx = player_angle.cos();
        let diry = player_angle.sin();

        // Perpendicular (para la base)
        let nx = -diry;
        let ny =  dirx;

        // Puntos del triángulo
        let tip  = Vector2::new(px + dirx * head_len,        py + diry * head_len);
        let base = Vector2::new(px - dirx * back_len,        py - diry * back_len);
        let p1   = Vector2::new(base.x + nx * base_half,     base.y + ny * base_half);
        let p2   = Vector2::new(base.x - nx * base_half,     base.y - ny * base_half);

        // Triángulo amarillo con borde negro fino
        d.draw_triangle(p1, p2, tip, Color::YELLOW);
        d.draw_triangle_lines(p1, p2, tip, Color::BLACK);

        // borde del minimapa + etiqueta
        d.draw_rectangle_lines(x0, y0, mw, mh, Color::WHITE);
        let label = if self.expanded { "M: contraer" } else { "M: expandir" };
        d.draw_text(label, x0 + 8, y0 + 8, 14, Color::WHITE);
    }
}

// ======================= HUD: Stats bajo el minimapa =======================
////////////////////////////////////////////////////////
// ======================= HUD: Stats bajo el minimapa =======================

fn format_time(mut secs: f32) -> String {
    if secs < 0.0 { secs = 0.0; }
    let total = secs as u32;
    let m = total / 60;
    let s = total % 60;
    format!("{m}:{s:02}")
}

/// Dibuja un icono circular con una letra dentro + el texto a la derecha.
/// Devuelve el ancho total consumido.
fn draw_icon_and_text(
    d: &mut RaylibDrawHandle,
    x: i32,
    y: i32,
    icon_char: &str,
    text: &str,
    font_size: i32,
) -> i32 {
    let gap = 8;                       // espacio icono ↔ texto
    let circle_r = (font_size as f32 * 0.55).max(8.0); // radio del icono
    let circle_d = (circle_r * 2.0) as i32;

    // midline vertical para alinear todo
    let cy = y + circle_d / 2;

    // círculo de icono
    let cx = x + circle_d / 2;
    d.draw_circle(cx, cy, circle_r, Color::new(50, 50, 50, 220));
    let iw = d.measure_text(icon_char, font_size - 2);
    d.draw_text(icon_char, cx - iw / 2, cy - (font_size - 2) / 2, font_size - 2, Color::WHITE);

    // texto
    let tx = x + circle_d + gap;
    let tw = d.measure_text(text, font_size);
    d.draw_text(text, tx, cy - font_size / 2, font_size, Color::WHITE);

    circle_d + gap + tw
}

/// Dibuja, bajo el minimapa, el tiempo, enemigos vivos y kills.
/// Se alinea al borde derecho del minimapa y **garantiza** que todo cabe sin salirse.
pub fn draw_top_right_stats(
    d: &mut RaylibDrawHandle,
    minimap: &Minimap,
    elapsed_secs: f32,
    enemies_left: usize,
    kills: u32,
) {
    let (mx, my, mw, mh) = minimap.bounds();

    // fila debajo del minimapa
    let margin = 8;
    let y = my + mh + margin;

    // prepara textos
    let font_size = 18;
    let t_text = format_time(elapsed_secs);
    let e_text = format!("{}", enemies_left);
    let k_text = format!("{}", kills);

    // medir anchos (sin dibujar) para posicionar de derecha a izquierda
    let gap_badges = 12;

    // para medir igual que draw_icon_and_text:
    // icono ocupa aprox circle_d = 2 * 0.55 * font_size
    let circle_d = (2.0 * 0.55 * font_size as f32).max(16.0) as i32;
    let icon_text_gap = 8;

    let w_time  = circle_d + icon_text_gap + d.measure_text(&t_text, font_size);
    let w_alive = circle_d + icon_text_gap + d.measure_text(&e_text, font_size);
    let w_kills = circle_d + icon_text_gap + d.measure_text(&k_text, font_size);

    // ancho total de la fila
    let total_w = w_time + w_alive + w_kills + 2 * gap_badges;

    // alineamos al borde derecho del minimapa y nos aseguramos de no salirnos
    let right = mx + mw;
    let mut start_x = right - total_w;
    if start_x < mx { start_x = mx; }                    // no sobrepasar el borde izquierdo del minimapa
    if start_x + total_w > SCREEN_W - margin {           // clamp al borde derecho de pantalla
        start_x = (SCREEN_W - margin - total_w).max(mx); // prioriza que quede bajo el minimapa
    }

    // dibujar izquierda → derecha
    let mut x = start_x;
    x += draw_icon_and_text(d, x, y, "T", &t_text, font_size);
    x += gap_badges;
    x += draw_icon_and_text(d, x, y, "E", &e_text, font_size);
    x += gap_badges;
    let _ = draw_icon_and_text(d, x, y, "K", &k_text, font_size);
}


// ===================== HUD: Barras de Escudo y Vida (bottom-center) =====================

fn draw_bar(
    d: &mut RaylibDrawHandle,
    x: i32,            // esquina izquierda
    y: i32,            // esquina superior
    w: i32,            // ancho total
    h: i32,            // alto
    value: i32,        // valor actual
    max_value: i32,    // máximo
    fill: Color,       // color de relleno
    bg: Color,         // color de fondo
    text: &str,        // texto a pintar encima
    icon_char: &str,   // símbolo a la izquierda
) {
    // fondo (barra completa)
    d.draw_rectangle_rounded(
        Rectangle { x: x as f32, y: y as f32, width: w as f32, height: h as f32 },
        0.35, 8, bg
    );

    // relleno proporcional
    let pct = if max_value > 0 { (value as f32 / max_value as f32).clamp(0.0, 1.0) } else { 0.0 };
    let fill_w = (w as f32 * pct) as i32;
    if fill_w > 0 {
        d.draw_rectangle_rounded(
            Rectangle { x: x as f32, y: y as f32, width: fill_w as f32, height: h as f32 },
            0.35, 8, fill
        );
    }

    // icono cuadrado a la izquierda (ligeramente más alto que la barra)
    let icon_pad = 6;
    let icon_w = h + icon_pad * 2;
    let icon_x = x - icon_w - 8; // 8px separador
    let icon_y = y - icon_pad;

    d.draw_rectangle_rounded(
        Rectangle { x: icon_x as f32, y: icon_y as f32, width: icon_w as f32, height: (h + icon_pad * 2) as f32 },
        0.35, 8, Color::new(30, 30, 30, 220)
    );

    let fs = (h as f32 * 0.8).max(14.0) as i32;
    let iw = d.measure_text(icon_char, fs);
    let ix_center = icon_x + icon_w / 2 - iw / 2;
    let iy_center = icon_y + (h + icon_pad * 2 - fs) / 2;
    d.draw_text(icon_char, ix_center, iy_center, fs, Color::WHITE);

    // texto (centrado verticalmente dentro de la barra)
    let text_fs = (h as f32 * 0.75).max(14.0) as i32;
    let tw = d.measure_text(text, text_fs);
    let tx = x + 10; // un poco de padding desde la izquierda de la barra
    let ty = y + (h - text_fs) / 2;

    // sombra suave para resaltar
    d.draw_text(text, tx + 1, ty + 1, text_fs, Color::new(0, 0, 0, 180));
    d.draw_text(text, tx,     ty,     text_fs, Color::WHITE);

    // números al extremo derecho (actual / máximo)
    let nums = format!("{}/{}", value.max(0), max_value.max(0));
    let nw = d.measure_text(&nums, text_fs);
    let nx = x + w - nw - 10;
    d.draw_text(&nums, nx + 1, ty + 1, text_fs, Color::new(0, 0, 0, 180));
    d.draw_text(&nums, nx,     ty,     text_fs, Color::WHITE);
}

/// Dibuja las barras (escudo arriba, vida abajo) centradas en la parte inferior.


//bara de slots de los objetos 
/// Colores de rareza (fondo del cuadro del arma)
fn rarity_bg(r: Rarity) -> Color {
    match r {
        Rarity::Common    => Color::new(120,120,120,220),
        Rarity::Uncommon  => Color::new( 55,180, 70,220),
        Rarity::Rare      => Color::new( 70,120,210,220),
        Rarity::Epic      => Color::new(140, 70,180,220),
        Rarity::Legendary => Color::new(210,170, 60,220),
    }
}
fn consumable_bg(c: ConsumableType) -> Color {
    match c {
        ConsumableType::HealthSmall | ConsumableType::HealthBig => Color::new(120,210,120,220),
        ConsumableType::ShieldSmall | ConsumableType::ShieldBig => Color::new(120,180,230,220),
    }
}
fn slot_short_text(item: &Item) -> (&'static str, Color) {
    match item {
        Item::Weapon(w, _) => {
            let t = match w.wtype {
                crate::types::WeaponType::Pistol         => "PST",
                crate::types::WeaponType::SMG            => "SMG",
                crate::types::WeaponType::Rifle          => "RFL",
                crate::types::WeaponType::Shotgun        => "ESC",
                crate::types::WeaponType::RocketLauncher => "RKT",
            };
            (t, Color::WHITE)
        }
        Item::Consumable(c) => {
            let t = match c {
                ConsumableType::HealthSmall => "HP+20",
                ConsumableType::HealthBig   => "HP+100",
                ConsumableType::ShieldSmall => "SH+25",
                ConsumableType::ShieldBig   => "SH+50",
            };
            (t, Color::WHITE)
        }
    }
}






pub fn draw_cooldown_circle(
    d: &mut RaylibDrawHandle,
    cx: i32,
    cy: i32,
    radius: i32,
    remaining: f32,
    total: f32,
    arc_color: Color,   // color del arco/progreso (p.ej. verde para vida, azul para escudo)
    bg_color: Color,    // color del fondo del círculo
) {
    if total <= 0.0 { return; }

    // Progreso de 0..1 (0 = recién empezó, 1 = terminó)
    let mut frac = 1.0 - (remaining / total);
    if frac.is_nan() { frac = 0.0; }
    frac = frac.clamp(0.0, 1.0);

    // Fondo circular
    d.draw_circle(cx, cy, radius as f32, bg_color);
    d.draw_circle_lines(cx, cy, radius as f32, Color::WHITE);

    // Anillo de fondo (gris) y arco de progreso
    let center = Vector2::new(cx as f32, cy as f32);
    let inner  = radius as f32 * 0.72;
    let outer  = radius as f32 * 0.96;

    // Anillo base
    d.draw_ring(center, inner, outer, 0.0, 360.0, 48, Color::new(255, 255, 255, 35));

    // Arco desde -90° (arriba) hasta sweep
    let start_deg = -90.0;
    let end_deg   = start_deg + 360.0 * frac;
    d.draw_ring(center, inner, outer, start_deg, end_deg, 48, arc_color);

    // Texto con segundos (1 decimal), centrado
    let secs = remaining.max(0.0);
    let text = format!("{:.1}", secs);
    let fs   = (radius as f32 * 0.8).max(14.0) as i32;
    let tw   = d.measure_text(&text, fs);
    let tx   = cx - tw / 2;
    let ty   = cy - fs / 2;
    d.draw_text(&text, tx + 1, ty + 1, fs, Color::new(0, 0, 0, 180));
    d.draw_text(&text, tx,     ty,     fs, Color::WHITE);
}

/// Helper de colores para consumible
pub fn cooldown_colors_for_health() -> (Color, Color) {
    // (arco, fondo)
    (Color::new(77, 209, 72, 255),  Color::new(10, 40, 15, 210)) // verde + fondo verdoso oscuro
}
pub fn cooldown_colors_for_shield() -> (Color, Color) {
    (Color::new(61, 153, 255, 255), Color::new(10, 25, 40, 210)) // azul + fondo azulado oscuro
}


// ===================== HUD: Contador de munición (centro-abajo) =====================

pub fn draw_ammo_center_bottom(
    d: &mut RaylibDrawHandle,
    mag: i32,      // balas en el cargador
    reserve: i32,  // balas en reserva
) {
    let margin_bottom = 28;
    let fs_left  = 44; // tamaño número izquierdo (mag)
    let fs_right = 44; // tamaño número derecho (reserva)
    let gap_nums = 18; // espacio entre números
    let gap_icon = 12; // espacio número derecho ↔ icono

    // textos
    let left_txt  = format!("{}", mag.max(0));
    let right_txt = format!("{}", reserve.max(0));

    // medir anchos
    let left_w  = d.measure_text(&left_txt, fs_left);
    let right_w = d.measure_text(&right_txt, fs_right);

    // tamaño del icono de balas (3 barritas)
    let bullet_w = 12;
    let bullet_h = 22;
    let bullet_gap = 4;
    let bullets_total_w = bullet_w * 3 + bullet_gap * 2;

    // ancho total
    let total_w = left_w + gap_nums + right_w + gap_icon + bullets_total_w;

    // posición base (centrado horizontal)
    let x0 = (crate::consts::SCREEN_W - total_w) / 2;
    let y_base = crate::consts::SCREEN_H - margin_bottom - fs_left; // línea base visual

    // ---- dibujar: número izquierdo (mag) con sombra suave ----
// número izquierdo (mag)
d.draw_text(&left_txt, x0 + 1, y_base + 1, fs_left, Color::new(0,0,0,180));
d.draw_text(&left_txt, x0,     y_base,     fs_left, Color::WHITE);

// --- SEPARADOR VERTICAL (en lugar del texto " / ") ---
let sep_center_x = x0 + left_w + gap_nums / 2;
let sep_w  = 4;                                    // ancho de la barra
let sep_h  = (fs_left as f32 * 0.85) as i32;       // alto aprox. del texto
let sep_x  = sep_center_x - sep_w / 2;
let sep_y  = y_base + (fs_left - sep_h) / 2;
// sombra suave
d.draw_rectangle(sep_x + 1, sep_y + 1, sep_w, sep_h, Color::new(0,0,0,160));
// barra blanca
d.draw_rectangle(sep_x, sep_y, sep_w, sep_h, Color::WHITE);

// número derecho (reserva)
let xr = x0 + left_w + gap_nums;
d.draw_text(&right_txt, xr + 1, y_base + 1, fs_right, Color::new(0,0,0,180));
d.draw_text(&right_txt, xr,     y_base,     fs_right, Color::WHITE);

// icono de balas a la derecha...
let xi = xr + right_w + gap_icon;
let cy = y_base + fs_right/2;
let top = cy - bullet_h/2;
let mut bx = xi;
for _ in 0..3 {
    d.draw_rectangle_rounded(
        Rectangle { x: bx as f32, y: top as f32, width: bullet_w as f32, height: bullet_h as f32 },
        0.55, 6, Color::WHITE
    );
    bx += bullet_w + bullet_gap;
}

}



// ====== BARRAS VIDA/ESCUDO: ahora ABAJO-IZQUIERDA (mantenemos el nombre) ======
pub fn draw_bottom_right_health_shield(
    d: &mut RaylibDrawHandle,
    hp: i32,
    shield: i32,
    max_hp: i32,
    max_shield: i32,
) {
    // Geometría
    let total_w = 420;   // ancho de la barra
    let bar_h   = 26;    // alto de cada barra
    let gap     = 10;    // separación entre barras
    let margin  = 18;    // margen del borde de pantalla

    // ⬇️ posición: esquina inferior-izquierda (cambiado)
    let x = margin;
    let y_bottom = SCREEN_H - margin - bar_h;       // VIDA abajo
    let y_top    = y_bottom - gap - bar_h;          // ESCUDO arriba

    // Colores
    let shield_fill = Color::new( 61, 153, 255, 255);
    let shield_bg   = Color::new( 40,  85, 140, 180);
    let hp_fill     = Color::new( 77, 209,  72, 255);
    let hp_bg       = Color::new( 45, 120,  40, 180);

    // Escudo (arriba)
    draw_bar(d, x, y_top, total_w, bar_h, shield, max_shield, shield_fill, shield_bg, "ESCUDO", "🛡");
    // Vida (abajo)
    draw_bar(d, x, y_bottom, total_w, bar_h, hp, max_hp, hp_fill, hp_bg, "VIDA", "＋");
}



// ====== SLOTS: ahora ABAJO-DERECHA (con ajuste hacia arriba para que se vean las teclas) ======
pub fn draw_slots_bar_bottom_left(
    d: &mut RaylibDrawHandle,
    slots: &[Option<SlotItem>; 5],
    selected: Option<usize>,
) {
    let tile: i32 = 72;        // tamaño de cuadro
    let gap:  i32 = 12;        // separación horizontal
    let count: usize = 6;      // 5 slots + mano vacía
    let margin: i32 = 18;      // margen desde el borde

    // Altura de la pastilla de tecla (depende del font size que usamos)
    let ky_fs: i32 = 18;       // font de la etiqueta
    let pill_h: i32 = ky_fs + 6;
    let label_gap: i32 = 6;    // espacio entre cuadro y pastilla

    // ⬇️ posición: esquina inferior-derecha
    let row_w: i32 = (count as i32) * tile + ((count as i32) - 1) * gap;
    let base_x: i32 = SCREEN_W - margin - row_w;

    // Subimos la fila para que la pastilla no se corte:
    // y + tile + label_gap + pill_h ≤ SCREEN_H - margin
    let base_y: i32 = SCREEN_H - margin - tile - (label_gap + pill_h);

    for i in 0..count {
        let x = base_x + (i as i32) * (tile + gap);
        let y = base_y;

        // === Fondo por tipo ===
        let mut bg = Color::new(30, 30, 30, 180);
        if i < 5 {
            if let Some(s) = &slots[i] {
                match &s.item {
                    Item::Weapon(w, _) => { bg = rarity_bg(w.rarity); }
                    Item::Consumable(c) => { bg = consumable_bg(*c); }
                }
            }
        } else {
            bg = Color::new(20, 20, 20, 180); // mano vacía
        }

        d.draw_rectangle_rounded(
            Rectangle { x: x as f32, y: y as f32, width: tile as f32, height: tile as f32 },
            0.2, 6, bg
        );

        let sel = match selected {
            Some(si) if i < 5 && si == i => true,
            None if i == 5 => true,
            _ => false,
        };
        let border_col = if sel { Color::WHITE } else { Color::new(120,120,120,200) };
        d.draw_rectangle_rounded_lines(
            Rectangle { x: x as f32, y: y as f32, width: tile as f32, height: tile as f32 },
            0.2, 6, border_col
        );

        // Texto placeholder / cantidades
        if i < 5 {
            if let Some(s) = &slots[i] {
                let (short, txt_col) = slot_short_text(&s.item);
                let fs = 18;
                let tw = d.measure_text(short, fs);
                d.draw_text(short, x + (tile - tw) / 2, y + tile/2 - fs, fs, txt_col);

                let fs2 = 16;
                let (sub, sub_col) = match &s.item {
                    Item::Weapon(w, ws) => (format!("{}/{}", ws.ammo_in_mag, w.mag_size), Color::new(240,240,240,255)),
                    Item::Consumable(_) => (format!("x{}", s.count), Color::new(240,240,240,255)),
                };
                let tw2 = d.measure_text(&sub, fs2);
                d.draw_text(&sub, x + (tile - tw2) / 2, y + tile/2 + 2, fs2, sub_col);
            }
        } else {
            let fs = 24;
            let tw = d.measure_text("—", fs);
            d.draw_text("—", x + (tile - tw) / 2, y + tile/2 - fs/2, fs, Color::new(200,200,200,220));
        }

        // Etiqueta tecla debajo (usa los mismos ky_fs/pill_h para que coincida con el cálculo superior)
        let key_label = match i { 0=>"1",1=>"2",2=>"3",3=>"4",4=>"5",_=>"0" };
        let kw = d.measure_text(key_label, ky_fs);
        let pill_w = kw + 14;
        let px = x + (tile - pill_w)/2;
        let py = y + tile + label_gap;

        d.draw_rectangle_rounded(
            Rectangle { x: px as f32, y: py as f32, width: pill_w as f32, height: pill_h as f32 },
            0.45, 6, Color::new(0,0,0,160)
        );
        d.draw_rectangle_rounded_lines(
            Rectangle { x: px as f32, y: py as f32, width: pill_w as f32, height: pill_h as f32 },
            0.45, 6, Color::new(200,200,200,220)
        );
        d.draw_text(
            key_label,
            px + (pill_w - kw)/2,
            py + (pill_h - ky_fs)/2 - 1,
            ky_fs,
            Color::WHITE
        );
    }
}

/// Círculo de cooldown de consumible (vida/escudo) centrado abajo,
/// colocado justo ENCIMA de los números de balas para no sobreponerse.
pub fn draw_consumable_cooldown_center_bottom(
    d: &mut RaylibDrawHandle,
    remaining: f32,           // segundos restantes (slot.cd)
    total: f32,               // segundos totales del consumo
    ctype: ConsumableType,    // para elegir colores (vida/escudo)
) {
    if total <= 0.0 || remaining <= 0.0 { return; }

    // Misma “línea base” que usamos en draw_ammo_center_bottom:
    // y_base = SCREEN_H - margin_bottom - fs_left
    let ammo_margin_bottom: i32 = 28;
    let ammo_fs_left: i32 = 44;
    let y_base = SCREEN_H - ammo_margin_bottom - ammo_fs_left;

    // Colocamos el círculo centrado, un poco por encima de la línea del ammo HUD
    let radius: i32 = 36;
    let gap_above_ammo: i32 = 14;
    let cx = SCREEN_W / 2;
    let cy = y_base - radius - gap_above_ammo;

    // Colores según tipo de consumible (vida/escudo)
    let (arc, bg) = match ctype {
        ConsumableType::HealthSmall | ConsumableType::HealthBig => cooldown_colors_for_health(),
        ConsumableType::ShieldSmall | ConsumableType::ShieldBig => cooldown_colors_for_shield(),
    };

    // Dibuja el círculo con texto de segundos (ya lo tenías implementado)
    draw_cooldown_circle(d, cx, cy, radius, remaining, total, arc, bg);
}


use crate::types::WeaponType;
use raylib::prelude::Texture2D;
pub struct WeaponHudTextures {
    pub pistol:   Texture2D,
    pub smg:      Texture2D,
    pub rifle:    Texture2D,
    pub shotgun:  Texture2D,
    pub rocket:   Texture2D,
}

impl WeaponHudTextures {
    pub fn tex(&self, wt: WeaponType) -> &Texture2D {
        match wt {
            WeaponType::Pistol         => &self.pistol,
            WeaponType::SMG            => &self.smg,
            WeaponType::Rifle          => &self.rifle,
            WeaponType::Shotgun        => &self.shotgun,
            WeaponType::RocketLauncher => &self.rocket,
        }
    }
}


/// Dibuja la textura del arma empuñada (solo “idle”), centrada abajo.
/// Se coloca **por encima** del HUD de balas para que no se encimen.
pub fn draw_held_weapon_center_bottom(
    d: &mut RaylibDrawHandle,
    tex: &Texture2D,
) {
    // Reservas del HUD de balas (idénticas a draw_ammo_center_bottom)
    let ammo_margin_bottom: i32 = 28;
    let ammo_fs_left: i32 = 44;
    let y_base_ammo = SCREEN_H - ammo_margin_bottom - ammo_fs_left;

    // Queremos que la imagen “descanse” un poco por encima del ammo HUD
    let gap = 12;

    // Limites máximos para no comer mucha pantalla
    let max_w = (SCREEN_W as f32 * 0.55) as i32;  // 55% del ancho
    let max_h = 220i32;                           // alto máximo aproximado

    let src_w = tex.width();
    let src_h = tex.height();
    if src_w == 0 || src_h == 0 { return; }

    // Escala con aspecto
    let sx = max_w as f32 / src_w as f32;
    let sy = max_h as f32 / src_h as f32;
    let scale = sx.min(sy).min(1.0); // no crecer de más si el PNG ya es pequeño

    let dst_w = (src_w as f32 * scale) as i32;
    let dst_h = (src_h as f32 * scale) as i32;

    let x = (SCREEN_W - dst_w) / 2;
    let y = y_base_ammo - gap - dst_h;

    // Sombra muy suave (opcional)
    let shadow_off = 2.0;
    d.draw_texture_pro(
        tex,
        Rectangle { x: 0.0, y: 0.0, width: src_w as f32, height: src_h as f32 },
        Rectangle { x: x as f32 + shadow_off, y: y as f32 + shadow_off, width: dst_w as f32, height: dst_h as f32 },
        Vector2::new(0.0, 0.0),
        0.0,
        Color::new(0, 0, 0, 90),
    );

    // Imagen principal
    d.draw_texture_pro(
        tex,
        Rectangle { x: 0.0, y: 0.0, width: src_w as f32, height: src_h as f32 },
        Rectangle { x: x as f32, y: y as f32, width: dst_w as f32, height: dst_h as f32 },
        Vector2::new(0.0, 0.0),
        0.0,
        Color::WHITE,
    );
}
