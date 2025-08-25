use raylib::prelude::*;
use raylib::consts::{MouseButton, KeyboardKey};
use crate::consts::{SCREEN_W, SCREEN_H};

/// Qué hacer desde la pantalla de victoria
pub enum VictoryAction {
    None,
    ToMenu,     // volver al menú
    Restart,    // volver a jugar (reiniciar partida)
}

pub struct VictoryScreen {
    hovered_menu: bool,
    hovered_restart: bool,
    // stats que mostramos
    elapsed_secs: f32,
    kills: u32,
}

impl VictoryScreen {
    pub fn new() -> Self {
        Self {
            hovered_menu: false,
            hovered_restart: false,
            elapsed_secs: 0.0,
            kills: 0,
        }
    }

    pub fn set_stats(&mut self, elapsed_secs: f32, kills: u32) {
        self.elapsed_secs = elapsed_secs.max(0.0);
        self.kills = kills;
    }

    fn btn_menu_rect(&self) -> (i32, i32, i32, i32) {
        let w = 240; let h = 56;
        let x = (SCREEN_W / 2) - w - 12; // a la izquierda del centro
        let y = (SCREEN_H * 2 / 3) - h/2;
        (x, y, w, h)
    }
    fn btn_restart_rect(&self) -> (i32, i32, i32, i32) {
        let w = 240; let h = 56;
        let x = (SCREEN_W / 2) + 12; // a la derecha del centro
        let y = (SCREEN_H * 2 / 3) - h/2;
        (x, y, w, h)
    }

    /// Manejar entrada (teclas/ratón). Llamar **antes** del begin_drawing.
    pub fn handle_input(&mut self, rl: &RaylibHandle) -> VictoryAction {
        let m = rl.get_mouse_position();

        let (mx, my, mw, mh) = self.btn_menu_rect();
        let (rx, ry, rw, rh) = self.btn_restart_rect();

        self.hovered_menu = m.x >= mx as f32 && m.x <= (mx+mw) as f32 && m.y >= my as f32 && m.y <= (my+mh) as f32;
        self.hovered_restart = m.x >= rx as f32 && m.x <= (rx+rw) as f32 && m.y >= ry as f32 && m.y <= (ry+rh) as f32;

        if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
            return VictoryAction::Restart;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_M) {
            return VictoryAction::ToMenu;
        }
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            if self.hovered_menu { return VictoryAction::ToMenu; }
            if self.hovered_restart { return VictoryAction::Restart; }
        }

        VictoryAction::None
    }

    /// Dibujo de la pantalla de victoria. Llamar **dentro** del begin_drawing.
    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        // Fondo
        d.draw_rectangle_gradient_v(
            0, 0, SCREEN_W, SCREEN_H,
            Color::new(20, 40, 20, 255),
            Color::new(50, 90, 50, 255),
        );

        // Título
        let title = "¡VICTORIA!";
        let fs_title = 56;
        let tw = d.measure_text(title, fs_title);
        d.draw_text(title, (SCREEN_W - tw)/2 + 2, 100 + 2, fs_title, Color::new(0,0,0,160));
        d.draw_text(title, (SCREEN_W - tw)/2,     100,     fs_title, Color::WHITE);

        // Stats
        let mins = (self.elapsed_secs as u32) / 60;
        let secs = (self.elapsed_secs as u32) % 60;
        let time_txt = format!("Tiempo: {}:{:02}", mins, secs);
        let kills_txt = format!("Eliminaciones: {}", self.kills);
        let fs_stats = 24;

        let tw_time  = d.measure_text(&time_txt, fs_stats);
        let tw_kills = d.measure_text(&kills_txt, fs_stats);
        d.draw_text(&time_txt,  (SCREEN_W - tw_time)/2,  180, fs_stats, Color::new(235,235,235,240));
        d.draw_text(&kills_txt, (SCREEN_W - tw_kills)/2, 210, fs_stats, Color::new(235,235,235,240));

        // Botones
        self.draw_button(d, self.btn_menu_rect(), "MENÚ (M)", self.hovered_menu);
        self.draw_button(d, self.btn_restart_rect(), "REINICIAR (ENTER)", self.hovered_restart);
    }

    fn draw_button(&self, d: &mut RaylibDrawHandle, rect: (i32,i32,i32,i32), label: &str, hovered: bool) {
        let (x, y, w, h) = rect;
        let round = 0.5; let segs = 10;
        let bg = if hovered { Color::new(80,160,80,255) } else { Color::new(60,120,60,255) };
        d.draw_rectangle_rounded(Rectangle{ x: x as f32, y: y as f32, width: w as f32, height: h as f32 }, round, segs, bg);
        d.draw_rectangle_rounded_lines(Rectangle{ x: x as f32, y: y as f32, width: w as f32, height: h as f32 }, round, segs, Color::WHITE);

        let fs = 24;
        let lw = d.measure_text(label, fs);
        d.draw_text(label, x + (w - lw)/2, y + (h - fs)/2, fs, Color::WHITE);
    }
}
