use crate::models::{Align, UIElements};
use raylib::{color::Color, prelude::*, text::RaylibFont};

/**the rectangle returned is the dimensions of the text drawn.*/
pub fn draw_text(d: &mut RaylibDrawHandle, text: &str, vertical: Align, horizontal: Align, font_size: i32, color: Color, offset: (i32, i32), ui_state: &UIElements) -> Rectangle {
    if let Some(font) = appropriate_font(ui_state, font_size) {
        let pos: (i32, i32, i32, i32) = calculate_dimensions_text(d, vertical, horizontal, offset, text, font_size, font);
        if ui_state.shadow {
            let opposite_color = Color::new(255 - color.r, 255 - color.g, 255 - color.b, 255);
            for i in -1..3 {
                for j in -1..3 {
                    d.draw_text_ex(font, text, Vector2::new(pos.0 as f32 + j as f32, pos.1 as f32 + i as f32), font_size as f32, 1., opposite_color);
                }
            }
        }
        d.draw_text_ex(font, text, Vector2::new(pos.0 as f32, pos.1 as f32), font_size as f32, 1., color);
        return rrect(pos.0, pos.1, pos.2.abs(), pos.3)
    }
    rrect(0, 0, 0, 0)
}

pub fn appropriate_font(ui_state: &UIElements, font_size: i32) -> Option<&Font> {
    ui_state
        .fonts
        .iter()
        .enumerate()
        .find(|&(i, _)| font_size <= i as i32 * ui_state.text_scale)
        .map(|(_, font)| font)
        .or_else(|| ui_state.fonts.last())
}

pub fn calc_text_size(text: &str, font_size: i32, font: &Font) -> (i32, i32) {
    let size = font.measure_text(text, font_size as f32, 1.);
    return (size.x as i32, size.y as i32);
}

pub fn calculate_dimensions_text(d: &mut RaylibDrawHandle, vertical: Align, horizontal: Align, offset: (i32, i32), text: &str, font_size: i32, font: &Font) -> (i32, i32, i32, i32) {
    let mut size = calc_text_size(text, font_size, font);
    let og_size = size.clone();
    match horizontal {
        Align::Start => size.0 = 0,
        Align::Middle => size.0 /= -2,
        Align::End => size.0 *= -1,
    }

    match vertical {
        Align::Start => size.1 = 0,
        Align::Middle => size.1 /= 2,
        Align::End => size.1 *= -1,
    }

    size.0 += offset.0;
    size.1 += offset.1;
    let p = calculate_position(d, vertical, horizontal, size);
    (p.0, p.1, og_size.0, og_size.1)
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
