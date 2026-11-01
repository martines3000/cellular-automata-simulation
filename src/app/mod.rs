mod theme;
mod world;

#[cfg(test)]
mod tests;

use eframe::{
    egui::{self, Button, Color32, RichText, Sense, Shape, Slider, Stroke},
    epi,
};
use egui::{pos2, vec2, Rect};
use world::cell::CellType;
use world::World;

pub struct App {
    running: bool,
    world: World,
    shapes: Vec<Shape>,
    pending_world_size: usize,
    fit_view: bool,
    show_grid: bool,
}

const MATERIALS: [(CellType, &str, &str, Color32); 6] = [
    (
        CellType::Water,
        "Water",
        "Flows, pools and carries floating wood.",
        theme::WATER,
    ),
    (
        CellType::Sand,
        "Sand",
        "Falls and settles into piles.",
        theme::SAND,
    ),
    (
        CellType::Dirt,
        "Dirt",
        "Solid terrain. Build walls and containers.",
        theme::DIRT,
    ),
    (
        CellType::Wood,
        "Wood",
        "Falls, floats on water and burns near fire.",
        theme::WOOD,
    ),
    (
        CellType::FireNormal,
        "Fire",
        "Ignites wood and leaves rising smoke.",
        theme::FIRE,
    ),
    (
        CellType::None,
        "Eraser",
        "Removes a cell. The outer border is protected.",
        theme::MUTED,
    ),
];

impl Default for App {
    fn default() -> Self {
        let num_of_blocks = 100;
        Self {
            running: false,
            world: World::new(num_of_blocks),
            shapes: Vec::new(),
            pending_world_size: num_of_blocks,
            fit_view: true,
            show_grid: false,
        }
    }
}

impl App {
    fn toggle_running(&mut self) {
        self.running = !self.running;
        self.world.reset_timing();
    }

    fn header(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("transport")
            .frame(
                egui::Frame::none()
                    .fill(theme::PANEL)
                    .margin(egui::style::Margin::same(16.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let (logo, _) = ui.allocate_exact_size(vec2(26.0, 30.0), Sense::hover());
                    for (row, width) in [24.0, 18.0, 12.0].into_iter().enumerate() {
                        let min = logo.min + vec2(0.0, row as f32 * 9.0 + 2.0);
                        ui.painter().rect_filled(
                            Rect::from_min_size(min, vec2(width, 5.0)),
                            1.0,
                            theme::ACCENT,
                        );
                    }
                    ui.vertical(|ui| {
                        ui.label(RichText::new("S T R A T A").size(21.0).strong());
                        ui.label(
                            RichText::new("ELEMENT SANDBOX")
                                .size(9.0)
                                .color(theme::MUTED),
                        );
                    });
                    ui.add_space(22.0);
                    let play = if self.running { "Pause" } else { "Play" };
                    if ui
                        .add_sized(
                            vec2(94.0, 36.0),
                            Button::new(RichText::new(play).strong().color(theme::BACKGROUND))
                                .fill(theme::ACCENT),
                        )
                        .on_hover_text("Play / pause · Space")
                        .clicked()
                    {
                        self.toggle_running();
                    }
                    if ui
                        .add_enabled(!self.running, Button::new("Step"))
                        .on_hover_text("Advance one simulation step while paused")
                        .clicked()
                    {
                        self.world.step_once();
                    }
                    ui.label(
                        RichText::new(if self.running { "LIVE" } else { "PAUSED" })
                            .monospace()
                            .size(11.0)
                            .color(if self.running {
                                theme::ACCENT
                            } else {
                                theme::MUTED
                            }),
                    );
                    ui.with_layout(egui::Layout::right_to_left(), |ui| {
                        ui.label(
                            RichText::new(format!(
                                "{} × {}",
                                self.world.data.ncols(),
                                self.world.data.nrows()
                            ))
                            .monospace()
                            .color(theme::MUTED),
                        );
                    });
                });
            });
    }

    fn toolbox(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("toolbox")
            .resizable(false)
            .default_width(264.0)
            .max_width(264.0)
            .frame(
                egui::Frame::none()
                    .fill(theme::PANEL)
                    .margin(egui::style::Margin::same(16.0)),
            )
            .show(ctx, |ui| {
                // Buttons own drags so a coalesced move/press cannot scroll them away.
                ui.spacing_mut().slider_width = 90.0;
                egui::ScrollArea::vertical().show(ui, |ui| {
                    section(ui, "01", "MATERIALS");
                    for pair in MATERIALS.chunks(2) {
                        ui.horizontal(|ui| {
                            let width = (ui.available_width() - ui.spacing().item_spacing.x) / 2.0;
                            for &(kind, name, description, color) in pair {
                                let selected = self.world.selected_cell_type == kind;
                                let label =
                                    RichText::new(name).size(14.0).color(theme::TEXT).strong();
                                let response = ui.add_sized(
                                    vec2(width, 43.0),
                                    Button::new(label).sense(Sense::click_and_drag()).stroke(
                                        Stroke::new(
                                            if selected { 2.0 } else { 1.0 },
                                            if selected { theme::ACCENT } else { theme::LINE },
                                        ),
                                    ),
                                );
                                ui.painter().rect_filled(
                                    Rect::from_min_size(
                                        response.rect.min + vec2(9.0, 12.0),
                                        vec2(4.0, 19.0),
                                    ),
                                    2.0,
                                    color,
                                );
                                if response.on_hover_text(description).clicked() {
                                    self.world.selected_cell_type = kind;
                                }
                            }
                        });
                    }
                    let (_, name, description, _) = MATERIALS
                        .iter()
                        .find(|(kind, _, _, _)| *kind == self.world.selected_cell_type)
                        .unwrap_or(&MATERIALS[0]);
                    egui::Frame::none()
                        .fill(theme::BACKGROUND)
                        .rounding(6.0)
                        .margin(egui::style::Margin::same(10.0))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label(RichText::new(*name).strong());
                            ui.label(RichText::new(*description).size(12.0).color(theme::MUTED));
                        });
                    ui.add_space(10.0);
                    section(ui, "02", "VIEW & SIMULATION");
                    if ui
                        .add(Button::new("Fit world").sense(Sense::click_and_drag()))
                        .on_hover_text("Show the whole world and reset pan")
                        .clicked()
                    {
                        self.fit_view = true;
                    }
                    if ui
                        .add(
                            Slider::new(&mut self.world.block_size, 0.25..=30.0)
                                .logarithmic(true)
                                .text("Zoom"),
                        )
                        .changed()
                    {
                        self.world.update_pos();
                        self.fit_view = false;
                    }
                    if ui
                        .add(Slider::new(&mut self.world.steps_per_second, 1..=360).text("Steps/s"))
                        .changed()
                    {
                        self.world.reset_timing();
                    }
                    ui.checkbox(&mut self.world.use_shift, "Fractional surfaces")
                        .on_hover_text(
                            "Draw water levels and floating wood at fractional cell heights",
                        );
                    ui.checkbox(&mut self.show_grid, "Guide grid");
                    egui::CollapsingHeader::new("Pan view").show(ui, |ui| {
                        let max = self.world.data.ncols() as f32;
                        let x = ui
                            .add(Slider::new(&mut self.world.shift.x, 0.0..=max).text("Horizontal"))
                            .changed();
                        let y = ui
                            .add(Slider::new(&mut self.world.shift.y, 0.0..=max).text("Vertical"))
                            .changed();
                        if x || y {
                            self.fit_view = false;
                            self.world.update_pos();
                        }
                    });
                    ui.add_space(10.0);
                    section(ui, "03", "WORLD");
                    ui.label(
                        RichText::new("Pause to edit the world.")
                            .size(12.0)
                            .color(theme::MUTED),
                    );
                    ui.add_enabled_ui(!self.running, |ui| {
                        ui.horizontal(|ui| {
                            egui::ComboBox::from_id_source("world_size")
                                .selected_text(format!(
                                    "{} × {}",
                                    self.pending_world_size, self.pending_world_size
                                ))
                                .show_ui(ui, |ui| {
                                    for size in [100, 256, 512, 1024] {
                                        ui.selectable_value(
                                            &mut self.pending_world_size,
                                            size,
                                            format!("{size} × {size}"),
                                        );
                                    }
                                });
                        });
                        if ui
                            .add_enabled(
                                self.pending_world_size != self.world.data.nrows(),
                                Button::new("Resize and clear").sense(Sense::click_and_drag()),
                            )
                            .on_hover_text("Replaces the world with an empty bordered grid")
                            .clicked()
                        {
                            self.world.resize_and_clear(self.pending_world_size);
                            self.fit_view = true;
                        }
                        ui.label(
                            RichText::new("Resizing clears all materials.")
                                .size(11.0)
                                .color(theme::MUTED),
                        );
                        egui::CollapsingHeader::new("Terrain & clearing").show(ui, |ui| {
                            ui.add(
                                Slider::new(&mut self.world.threshold, 0.0..=1.0).text("Density"),
                            );
                            if ui
                                .add(Button::new("Generate terrain").sense(Sense::click_and_drag()))
                                .clicked()
                            {
                                self.world.rand_generate();
                            }
                            ui.horizontal(|ui| {
                                if ui
                                    .add(Button::new("Smooth").sense(Sense::click_and_drag()))
                                    .clicked()
                                {
                                    self.world.smooth();
                                }
                                if ui
                                    .add(Button::new("Clear world").sense(Sense::click_and_drag()))
                                    .clicked()
                                {
                                    self.world.clear();
                                }
                            });
                        });
                    });
                });
            });
    }

    fn canvas(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(theme::BACKGROUND)
                    .margin(egui::style::Margin::same(16.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("THE SANDBOX")
                            .size(11.0)
                            .strong()
                            .color(theme::MUTED),
                    );
                    ui.with_layout(egui::Layout::right_to_left(), |ui| {
                        ui.label(
                            RichText::new("Click or drag to place material")
                                .size(11.0)
                                .color(theme::MUTED),
                        );
                    });
                });
                ui.add_space(4.0);
                let (outer, _) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
                ui.painter().rect_filled(outer, 10.0, theme::LINE);
                let rect = outer.shrink(1.0);
                let painter = egui::Painter::new(ui.ctx().clone(), ui.layer_id(), rect);
                painter.rect_filled(rect, 9.0, theme::CANVAS);
                if self.fit_view && rect.is_positive() {
                    let size = self.world.data.ncols() as f32;
                    self.world.block_size =
                        (rect.width().min(rect.height()) / size).clamp(0.25, 30.0);
                    self.world.shift = vec2(0.0, 0.0);
                    self.world.update_pos();
                }
                let response = ui.interact(rect, ui.id().with("paint"), Sense::click_and_drag());
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
                // egui starts dragging on press; don't add the same click on release.
                if response.dragged_by(egui::PointerButton::Primary)
                    || (response.clicked() && primary_pressed)
                {
                    self.world
                        .transform_cell(response.interact_pointer_pos(), rect);
                }
                if self.running {
                    self.world.update();
                }
                if self.show_grid {
                    let spacing = (self.world.block_size * 10.0).max(8.0);
                    let start = rect.min
                        + vec2(
                            self.world.pos_shift.x.rem_euclid(spacing),
                            self.world.pos_shift.y.rem_euclid(spacing),
                        );
                    for x in 0..=((rect.width() / spacing).ceil() as usize) {
                        let x = start.x + x as f32 * spacing;
                        painter.line_segment(
                            [pos2(x, rect.top()), pos2(x, rect.bottom())],
                            Stroke::new(0.5, theme::SURFACE),
                        );
                    }
                    for y in 0..=((rect.height() / spacing).ceil() as usize) {
                        let y = start.y + y as f32 * spacing;
                        painter.line_segment(
                            [pos2(rect.left(), y), pos2(rect.right(), y)],
                            Stroke::new(0.5, theme::SURFACE),
                        );
                    }
                }
                self.shapes.clear();
                self.world.gen_shapes(&mut self.shapes, rect);
                for mut shape in self.shapes.drain(..) {
                    if let Shape::Rect(cell) = &mut shape {
                        cell.fill = theme::scene_color(cell.fill);
                    }
                    painter.add(shape);
                }
                if response.hovered() {
                    if let Some(pos) = response.hover_pos() {
                        let origin = rect.min + self.world.pos_shift;
                        let cell = ((pos - origin) / self.world.block_size).floor();
                        if self.world.bounds_valid(cell) {
                            let cell_rect = Rect::from_min_size(
                                origin + cell * self.world.block_size,
                                vec2(self.world.block_size, self.world.block_size),
                            );
                            painter.rect_stroke(cell_rect, 0.0, Stroke::new(1.0, theme::ACCENT));
                        }
                    }
                }
            });
    }
}

fn section(ui: &mut egui::Ui, number: &str, title: &str) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(number)
                .size(10.0)
                .monospace()
                .color(theme::ACCENT),
        );
        ui.label(RichText::new(title).size(11.0).strong().color(theme::MUTED));
    });
    ui.add_space(3.0);
}

impl epi::App for App {
    fn name(&self) -> &str {
        "Strata · Element Sandbox"
    }

    fn setup(
        &mut self,
        ctx: &egui::Context,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        theme::install(ctx);
        self.world.add_border();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        if !ctx.wants_keyboard_input() && ctx.input().key_pressed(egui::Key::Space) {
            self.toggle_running();
        }
        self.header(ctx);
        egui::TopBottomPanel::bottom("status")
            .frame(
                egui::Frame::none()
                    .fill(theme::PANEL)
                    .margin(egui::style::Margin::symmetric(16.0, 8.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("SPACE")
                            .monospace()
                            .size(10.0)
                            .color(theme::ACCENT),
                    );
                    ui.label(RichText::new("play / pause").size(11.0).color(theme::MUTED));
                    ui.separator();
                    ui.label(
                        RichText::new("Experiment. Build. Let it flow.")
                            .size(11.0)
                            .color(theme::MUTED),
                    );
                });
            });
        self.toolbox(ctx);
        self.canvas(ctx);
        if self.running {
            ctx.request_repaint();
        }
    }
}
