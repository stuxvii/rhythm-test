use crate::models::{Align, UIElements};
use raylib::{color::Color, prelude::*, text::RaylibFont};

pub fn draw_text(d: &mut RaylibDrawHandle, text: &str, vertical: Align, horizontal: Align, font_size: i32, color: Color, offset: (i32, i32), ui_state: &UIElements) {
    let mut n: usize = ui_state.fonts.len();
    for i in 0..ui_state.fonts.len() {
        if font_size <= i as i32 * ui_state.text_scale {
            n = i;
            break;
        } else {
            continue;
        }
    }
    if let Some(font) = ui_state.fonts.get(n) {
        let text_width = font.measure_text(text, font_size as f32, 1.).x as i32;
        let mut x = match horizontal {
            Align::Start => 0,
            Align::Middle => (d.get_screen_width() / 2) - (text_width / 2),
            Align::End => d.get_screen_width() - text_width,
        } as f32;
        let mut y = match vertical {
            Align::Start => 0,
            Align::Middle => (d.get_screen_height() / 2) - (font_size / 2),
            Align::End => d.get_screen_height() - font_size,
        } as f32;
        x += offset.0 as f32;
        y += offset.1 as f32;
        if ui_state.shadow {
            let opposite_color = Color::new(255 - color.r, 255 - color.g, 255 - color.b, 255);
            for i in -3..1 {
                for j in -1..3 {
                    d.draw_text_ex(font, text, Vector2::new(x + j as f32, y + i as f32), font_size as f32, 1., opposite_color);
                }
            }
        }
        d.draw_text_ex(font, text, Vector2::new(x, y - 2.), font_size as f32, 1., color);
    }
}

pub fn calc_size(text: &str, ui_state: &UIElements, font_size: i32) -> i32 {
    let mut n: usize = ui_state.fonts.len();
    for i in 0..ui_state.fonts.len() {
        if font_size <= i as i32 * ui_state.text_scale {
            n = i;
            break;
        } else {
            continue;
        }
    }
    if let Some(font) = ui_state.fonts.get(n) {
        return font.measure_text(text, font_size as f32, 1.).x as i32;
    }
    0
}

pub fn calculate_position(d: &mut RaylibDrawHandle, vertical: Align, horizontal: Align, offset: (i32, i32)) -> (i32, i32) {
    let mut x = match horizontal {
        Align::Start => 0,
        Align::Middle => d.get_screen_width() / 2,
        Align::End => d.get_screen_width(),
    };
    let mut y = match vertical {
        Align::Start => 0,
        Align::Middle => d.get_screen_height() / 2,
        Align::End => d.get_screen_height(),
    };
    x += offset.0;
    y += offset.1;
    (x, y)
}
