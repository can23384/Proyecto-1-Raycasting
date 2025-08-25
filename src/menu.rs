use raylib::prelude::*;
use crate::consts::{SCREEN_W, SCREEN_H};
use raylib::consts::MouseButton;

/// Acciones posibles del menú
pub enum MenuAction {
    None,
    Start,
}

pub struct Menu {
    title: String,
    btn_text: String,
    btn_size: (i32, i32),
    hovered: bool,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            title: "PROYECTO RAYCAST".to_string(),
            btn_text: "INICIAR".to_string(),
            btn_size: (260, 64),
            hovered: false,
        }
    }

    /// Procesa entrada del usuario (teclado/ratón). Llama **antes** de begin_drawing.
    pub fn handle_input(&mut self, rl: &RaylibHandle) -> MenuAction {
        let (bx, by, bw, bh) = self.button_rect();
        let m = rl.get_mouse_position();

        self.hovered = m.x >= bx as f32 && m.x <= (bx + bw) as f32 &&
                       m.y >= by as f32 && m.y <= (by + bh) as f32;

        if self.hovered && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            return MenuAction::Start;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_ENTER) || rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            return MenuAction::Start;
        }
        MenuAction::None
    }

    /// Dibuja la pantalla de inicio. Llama dentro del begin_drawing.
    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        // Fondo
        d.draw_rectangle_gradient_v(
            0, 0, SCREEN_W, SCREEN_H,
            Color::new(15, 15, 30, 255),
            Color::new(35, 35, 70, 255),
        );

        // Título
        let title_fs = 48;
        let tw = d.measure_text(&self.title, title_fs);
        d.draw_text(&self.title, (SCREEN_W - tw)/2 + 2, 90 + 2, title_fs, Color::new(0,0,0,150));
        d.draw_text(&self.title, (SCREEN_W - tw)/2,      90,     title_fs, Color::WHITE);

        // Subtítulo / tips
        let sub = "W/S: avanzar | A/D: girar | 1..5/0: slots | M: minimapa";
        let sub_fs = 20;
        let sw = d.measure_text(sub, sub_fs);
        d.draw_text(sub, (SCREEN_W - sw)/2, 150, sub_fs, Color::new(220,220,220,230));

        // Botón
        let (bx, by, bw, bh) = self.button_rect();
        let round = 0.5;
        let segs = 12;
        let bg = if self.hovered { Color::new(70,130,250,255) } else { Color::new(50,90,200,255) };
        d.draw_rectangle_rounded(Rectangle{ x: bx as f32, y: by as f32, width: bw as f32, height: bh as f32 }, round, segs, bg);
        d.draw_rectangle_rounded_lines(Rectangle{ x: bx as f32, y: by as f32, width: bw as f32, height: bh as f32 }, round, segs, Color::WHITE);

        let fs = 28;
        let btw = d.measure_text(&self.btn_text, fs);
        d.draw_text(&self.btn_text, bx + (bw - btw)/2, by + (bh - fs)/2, fs, Color::WHITE);

        // Hint
        let hint = "ENTER o CLICK para iniciar";
        let hf = 18;
        let hw = d.measure_text(hint, hf);
        d.draw_text(hint, (SCREEN_W - hw)/2, by + bh + 14, hf, Color::new(230,230,230,220));
    }

    /// Rectángulo del botón centrado en la parte inferior media.
    fn button_rect(&self) -> (i32, i32, i32, i32) {
        let (bw, bh) = self.btn_size;
        let x = (SCREEN_W - bw) / 2;
        let y = (SCREEN_H * 2 / 3) - (bh / 2);
        (x, y, bw, bh)
    }
}
