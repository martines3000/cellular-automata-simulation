use super::{theme, world::cell::CellType, World};
use egui::{pos2, vec2, Color32, ColorImage, Rect};

// One texel per visible cell: use only for non-fractional, zoomed-out overviews.
// The pinned Glow backend filters linearly; close views retain crisp geometry.
pub(super) fn overview(world: &World, viewport: Rect) -> Option<(Rect, ColorImage)> {
    let scale = world.block_size;
    if !scale.is_finite()
        || scale <= 0.0
        || !world.pos_shift.is_finite()
        || !viewport.is_finite()
        || !viewport.is_positive()
    {
        return None;
    }
    let (rows, cols) = world.data.dim();
    let start_x = (-world.pos_shift.x / scale).floor().clamp(0.0, cols as f32) as usize;
    let start_y = (-world.pos_shift.y / scale).floor().clamp(0.0, rows as f32) as usize;
    let end_x = ((viewport.width() - world.pos_shift.x) / scale)
        .ceil()
        .clamp(0.0, cols as f32) as usize;
    let end_y = ((viewport.height() - world.pos_shift.y) / scale)
        .ceil()
        .clamp(0.0, rows as f32) as usize;
    if end_x <= start_x || end_y <= start_y {
        return None;
    }
    let mut image = ColorImage::new([end_x - start_x, end_y - start_y], Color32::TRANSPARENT);
    for y in start_y..end_y {
        for x in start_x..end_x {
            let cell = world.data[[y, x]];
            let above_water = y > 0 && world.data[[y - 1, x]].cell_type == CellType::Water;
            let color = if cell.cell_type == CellType::Water
                || (cell.cell_type == CellType::None && above_water)
            {
                if cell.mass > super::world::MIN_DRAW {
                    theme::WATER
                } else {
                    Color32::TRANSPARENT
                }
            } else if cell.cell_type == CellType::None {
                Color32::TRANSPARENT
            } else {
                theme::scene_color(cell.color())
            };
            image.pixels[(y - start_y) * image.size[0] + x - start_x] = color;
        }
    }
    let origin = viewport.min + world.pos_shift;
    Some((
        Rect::from_min_max(
            origin + vec2(start_x as f32, start_y as f32) * scale,
            origin + vec2(end_x as f32, end_y as f32) * scale,
        ),
        image,
    ))
}

pub(super) const UV: Rect = Rect {
    min: pos2(0.0, 0.0),
    max: pos2(1.0, 1.0),
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::world::cell::Cell;
    use egui::Shape;

    #[test]
    fn overview_matches_rectangle_colors_at_cell_centers_with_pan_and_clipping() {
        for shift in [vec2(0.0, 0.0), vec2(1.3, 2.7), vec2(10.0, 10.0)] {
            let mut world = World::new(12);
            world.add_border();
            world.block_size = 2.0;
            world.shift = shift;
            world.update_pos();
            let types = [
                CellType::Water,
                CellType::None,
                CellType::Ice,
                CellType::Snow,
                CellType::Steam,
                CellType::Sand,
                CellType::Wood,
                CellType::FireNormal,
            ];
            for y in 1..11 {
                for x in 1..11 {
                    world.data[[y, x]] = Cell {
                        cell_type: types[(x + y) % types.len()],
                        mass: if x % 3 == 0 { 0.005 } else { 1.0 },
                    };
                }
            }
            let view = Rect::from_min_size(pos2(50.0, 20.0), vec2(15.0, 13.0));
            let (rect, image) = overview(&world, view).unwrap();
            let mut shapes = vec![];
            world.gen_shapes(&mut shapes, view);
            for y in 0..image.size[1] {
                for x in 0..image.size[0] {
                    let p = rect.min + vec2(x as f32 + 0.5, y as f32 + 0.5) * world.block_size;
                    if !view.contains(p) {
                        continue;
                    }
                    let expected = shapes
                        .iter()
                        .rev()
                        .find_map(|shape| match shape {
                            Shape::Rect(r) if r.rect.contains(p) => {
                                Some(theme::scene_color(r.fill))
                            }
                            _ => None,
                        })
                        .unwrap_or(Color32::TRANSPARENT);
                    assert_eq!(image.pixels[y * image.size[0] + x], expected);
                }
            }
        }
    }

    #[test]
    fn overview_rejects_invalid_or_offscreen_views() {
        let mut world = World::new(7);
        let view = Rect::from_min_size(pos2(0.0, 0.0), vec2(10.0, 10.0));
        for scale in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            world.block_size = scale;
            assert!(overview(&world, view).is_none());
        }
        world.block_size = 1.0;
        world.shift = vec2(100.0, 100.0);
        world.update_pos();
        assert!(overview(&world, view).is_none());
    }
}

pub(super) fn fire_glow(shapes: &[egui::Shape], painter: &egui::Painter, scale: f32) {
    if scale < 4.0 {
        return;
    }
    for r in shapes
        .iter()
        .filter_map(|shape| match shape {
            egui::Shape::Rect(r)
                if r.fill == egui::Color32::LIGHT_RED || r.fill == egui::Color32::DARK_RED =>
            {
                Some(r)
            }
            _ => None,
        })
        .take(128)
    {
        painter.rect_filled(
            r.rect.expand((scale * 0.3).min(4.0)),
            2.0,
            egui::Color32::from_rgba_unmultiplied(255, 121, 59, 28),
        );
    }
}

pub(super) fn surface_accent(shape: &egui::Shape, scale: f32) -> Option<egui::Shape> {
    if scale < 4.0 {
        return None;
    }
    let egui::Shape::Rect(r) = shape else {
        return None;
    };
    let color = if r.fill == theme::WATER && r.rect.height() < scale - 0.01 {
        egui::Color32::from_rgba_unmultiplied(177, 226, 255, 150)
    } else if r.fill == super::world::cell::ICE_COLOR {
        egui::Color32::from_rgba_unmultiplied(224, 251, 255, 115)
    } else if r.fill == super::world::cell::SNOW_COLOR {
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 100)
    } else {
        return None;
    };
    Some(egui::Shape::line_segment(
        [r.rect.left_top(), r.rect.right_top()],
        egui::Stroke::new(0.7, color),
    ))
}

#[cfg(test)]
mod accent_tests {
    use super::*;
    use egui::{Color32, Shape};

    #[test]
    fn accents_are_static_close_view_only_and_do_not_decorate_empty_space() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(5.0, 5.0));
        let ice = Shape::rect_filled(rect, 0.0, super::super::world::cell::ICE_COLOR);
        assert!(surface_accent(&ice, 2.0).is_none());
        assert!(surface_accent(&ice, 5.0).is_some());
        assert!(surface_accent(&Shape::rect_filled(rect, 0.0, Color32::WHITE), 5.0).is_none());
    }
}
