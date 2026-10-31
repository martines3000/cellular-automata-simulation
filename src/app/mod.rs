mod world;

#[cfg(test)]
mod tests;

use eframe::{
    egui::{self, Button, Painter, Sense, Slider, SliderOrientation},
    epaint::{vec2, Pos2},
    epi::{self},
};

use egui::{Color32, Rounding, Shape};
use world::cell::CellType;
use world::World;

pub struct App {
    running: bool,
    world: World,
    shapes: Vec<Shape>,
}

impl Default for App {
    fn default() -> Self {
        let block_size = 1.0;
        let num_of_blocks = (100.0 / block_size) as usize;
        Self {
            running: false,
            world: World::new(num_of_blocks),
            shapes: Vec::new(),
        }
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        "Vaja 2"
    }

    fn setup(
        &mut self,
        _ctx: &egui::Context,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        self.world.add_border();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        let Self {
            world,
            running,
            shapes,
        } = self;

        egui::Window::new("Menu").resizable(false).show(ctx, |ui| {
            if ui
                .add(
                    Slider::new(&mut world.block_size, 5.0..=30.0)
                        .step_by(1.0)
                        .orientation(SliderOrientation::Horizontal)
                        .text("Block size"),
                )
                .changed()
            {
                world.update_pos();
            }

            ui.separator();

            ui.horizontal(|ui| {
                if ui.add(Button::new("Toggle")).clicked() {
                    *running = !*running;
                    world.reset_timing();
                }

                if ui
                    .add_enabled(!*running, Button::new("Generate new"))
                    .clicked()
                {
                    world.rand_generate();
                }

                if ui.add_enabled(!*running, Button::new("Clear")).clicked() {
                    world.clear();
                }
                if ui.add_enabled(!*running, Button::new("Smooth")).clicked() {
                    world.smooth();
                }

                if ui.add(Button::new("Toggle shift")).clicked() {
                    world.toggle_shift();
                }
            });

            if ui
                .add(
                    Slider::new(&mut world.steps_per_second, 1..=360)
                        .step_by(1.0)
                        .orientation(SliderOrientation::Horizontal)
                        .text("Steps/s"),
                )
                .changed()
            {
                world.reset_timing();
            }

            if ui
                .add(
                    Slider::new(&mut world.shift.x, 0.0..=100.0)
                        .step_by(1.0)
                        .orientation(SliderOrientation::Horizontal)
                        .text("Shift x"),
                )
                .changed()
            {
                world.update_pos();
            }

            if ui
                .add(
                    Slider::new(&mut world.shift.y, 0.0..=100.0)
                        .step_by(1.0)
                        .orientation(SliderOrientation::Horizontal)
                        .text("Shift y"),
                )
                .changed()
            {
                world.update_pos();
            }

            ui.separator();

            ui.add(
                Slider::new(&mut world.threshold, 0.0..=1.0)
                    .step_by(0.01)
                    .orientation(SliderOrientation::Horizontal)
                    .text("Threshold"),
            );

            ui.separator();

            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        world.selected_cell_type.ne(&CellType::Water),
                        Button::new("Water"),
                    )
                    .clicked()
                {
                    world.selected_cell_type = CellType::Water;
                }

                if ui
                    .add_enabled(
                        world.selected_cell_type.ne(&CellType::Dirt),
                        Button::new("Dirt"),
                    )
                    .clicked()
                {
                    world.selected_cell_type = CellType::Dirt;
                }

                if ui
                    .add_enabled(
                        world.selected_cell_type.ne(&CellType::Sand),
                        Button::new("Sand"),
                    )
                    .clicked()
                {
                    world.selected_cell_type = CellType::Sand;
                }

                if ui
                    .add_enabled(
                        world.selected_cell_type.ne(&CellType::Wood),
                        Button::new("Wood"),
                    )
                    .clicked()
                {
                    world.selected_cell_type = CellType::Wood;
                }

                if ui
                    .add_enabled(
                        world.selected_cell_type.ne(&CellType::FireNormal),
                        Button::new("Fire"),
                    )
                    .clicked()
                {
                    world.selected_cell_type = CellType::FireNormal;
                }

                if ui
                    .add_enabled(
                        world.selected_cell_type.ne(&CellType::None),
                        Button::new("Empty"),
                    )
                    .clicked()
                {
                    world.selected_cell_type = CellType::None;
                }
            });
        });

        let display = egui::Window::new("Display")
            .default_size(vec2(600.0, 600.0))
            .default_pos(Pos2 { x: 600.0, y: 200.0 })
            .show(ctx, |ui| {
                let painter = Painter::new(
                    ui.ctx().clone(),
                    ui.layer_id(),
                    ui.available_rect_before_wrap(),
                );

                ui.expand_to_include_rect(painter.clip_rect());
                let rect = painter.clip_rect();
                shapes.clear();

                let response = ui.interact(rect, ui.id(), Sense::click_and_drag());
                let primary_pressed = ui.input().events.iter().any(|event| {
                    matches!(
                        event,
                        egui::Event::PointerButton {
                            button: egui::PointerButton::Primary,
                            pressed: true,
                            ..
                        }
                    )
                });
                // egui 0.17 starts dragging on press; don't also paint that click on release.
                if response.dragged_by(egui::PointerButton::Primary)
                    || (response.clicked() && primary_pressed)
                {
                    world.transform_cell(response.interact_pointer_pos(), rect);
                }

                if *running {
                    world.update();
                }

                shapes.push(Shape::rect_filled(rect, Rounding::none(), Color32::WHITE));

                world.gen_shapes(shapes, rect);

                for shape in shapes.drain(..) {
                    painter.add(shape);
                }
            });

        if *running {
            // Collapsed windows skip their body, but the simulation must keep advancing.
            if display.and_then(|response| response.inner).is_none() {
                world.update();
            }
            ctx.request_repaint();
        }
    }
}
