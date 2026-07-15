use crate::models::{Align, AppState, UIElements, rrect};
use raylib::{color::Color, ffi::Rectangle, prelude::*, text::RaylibFont};

/**the rectangle returned is the dimensions of the text drawn.*/
pub fn draw_text(
    d: &mut RaylibDrawHandle,
    text: &str,
    vertical: Align,
    horizontal: Align,
    font_size: i32,
    color: Color,
    offset: (i32, i32),
    ui_state: &UIElements,
) -> Rectangle {
    let f = if let Some(font) = appropriate_font(ui_state, font_size) {
        font
    } else {
        unsafe { &Font::from_raw(*d.get_font_default()) }
    };
    let pos = calculate_dimensions_text(d, vertical, horizontal, offset, text, font_size, f);
    if ui_state.shadow {
        // let o_clr = Color::new(255 - color.r, 255 - color.g, 255 - color.b, 255);
        let o_clr = Color::BLACK;
        for i in -1..2 {
            for j in -1..2 {
                let p = Vector2::new(pos.x + j as f32, pos.y + i as f32);
                d.draw_text_ex(f, text, p, font_size as f32, 1., o_clr);
            }
        }
    }
    d.draw_text_ex(f, text, Vector2::new(pos.x, pos.y), font_size as f32, 1., color);
    pos
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

pub fn calculate_dimensions_text(
    d: &mut RaylibDrawHandle,
    vertical: Align,
    horizontal: Align,
    offset: (i32, i32),
    text: &str,
    font_size: i32,
    font: &Font,
) -> Rectangle {
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
    rrect(p.0, p.1, og_size.0.abs(), og_size.1)
}

pub fn calculate_position(d: &mut RaylibDrawHandle, vertical: Align, horizontal: Align, offset: (i32, i32)) -> (i32, i32) {
    let mut x = match horizontal {
        Align::Start => 0,
        Align::Middle => d.get_render_width() / 2,
        Align::End => d.get_render_width(),
    };

    let mut y = match vertical {
        Align::Start => 0,
        Align::Middle => d.get_render_height() / 2,
        Align::End => d.get_render_height(),
    };
    x += offset.0;
    y += offset.1;
    (x, y)
}

pub fn draw_list_item(
    state: &AppState,
    d: &mut RaylibDrawHandle<'_>,
    visual_offset: usize,
    label: &str,
    banner: &Option<Texture2D>,
    is_selected: bool,
) -> bool {
    let mut rect = rrect(0, 0, state.viewport.w / 2, 60);
    let mut position = calculate_position(
        d,
        Align::Start,
        Align::End,
        (-rect.width as i32, (rect.height + 5.) as i32 * visual_offset as i32),
    );
    rect.x = position.0 as f32;
    if is_selected {
        rect.x -= 10.;
    }
    rect.y = position.1 as f32;
    d.draw_rectangle_rec(rect, Color::DIMGRAY);
    position.1 += rect.height as i32 / 2;
    position.1 -= 5;

    rect.width /= 2.;
    rect.x += rect.width;

    if let Some(texture) = banner {
        d.draw_texture_pro(
            texture,
            rrect(0, 0, texture.width, texture.height),
            rect,
            Vector2::new(0., 0.),
            0.,
            Color::WHITE,
        );
    } else {
        d.draw_rectangle_gradient_ex(rect, Color::BLANK, Color::BLANK, Color::WHITE, Color::WHITE);
    }
    draw_text(d, &label, Align::Start, Align::Start, 15, state.ui.fg, position, &state.ui);
    is_selected && d.is_key_pressed(state.config.keybinds.confirm)
}

pub fn center_crop_fit(content_size: Vector2, screen_size: Vector2) -> Rectangle {
    let scale_x = screen_size.x / content_size.x;
    let scale_y = screen_size.y / content_size.y;
    let scale = scale_x.max(scale_y);
    let width = content_size.x * scale;
    let height = content_size.y * scale;
    let x = (screen_size.x - width) / 2.0;
    let y = (screen_size.y - height) / 2.0;

    Rectangle { x, y, width, height }
}
