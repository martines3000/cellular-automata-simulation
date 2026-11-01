mod brush;
mod render;
mod theme;
mod world;

#[cfg(test)]
mod tests;

use eframe::{
    egui::{self, Button, Color32, RichText, Sense, Shape, Slider, Stroke},
    epi,
};
use egui::{pos2, vec2, Rect};
use world::cell::{CellType, ICE_COLOR, SNOW_COLOR, STEAM_COLOR};
use world::labs::Lab;
use world::{PhaseTool, World};

pub struct App {
    running: bool,
    world: World,
    shapes: Vec<Shape>,
    pending_world_size: usize,
    fit_view: bool,
    show_grid: bool,
    phase_tool: Option<PhaseTool>,
    brush: brush::Brushes,
    lab: Lab,
    highlights: bool,
    fast_overview: bool,
    texture: Option<egui::TextureHandle>,
    frame_ms: f32,
    simulation_ms: f32,
    render_ms: f32,
    rendered_shapes: usize,
}

const MATERIALS: [(CellType, &str, &str, Color32); 9] = [
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
        CellType::Ice,
        "Ice",
        "Falls through air, floats in water. Fire or Melt returns its water.",
        ICE_COLOR,
    ),
    (
        CellType::Snow,
        "Snow",
        "Light powder. Melts beside water or fire; one water unit per cell.",
        SNOW_COLOR,
    ),
    (
        CellType::Steam,
        "Steam",
        "Rises and drifts without expiring. Ice, Snow or Condense returns water.",
        STEAM_COLOR,
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
        let mut world = World::new(num_of_blocks);
        world.load_lab(Lab::FloatingIce);
        Self {
            running: false,
            world,
            shapes: Vec::new(),
            pending_world_size: num_of_blocks,
            fit_view: true,
            show_grid: false,
            phase_tool: None,
            brush: brush::Brushes::default(),
            lab: Lab::FloatingIce,
            highlights: true,
            fast_overview: true,
            texture: None,
            frame_ms: 0.0,
            simulation_ms: 0.0,
            render_ms: 0.0,
            rendered_shapes: 0,
        }
    }
}

impl App {
    fn toggle_running(&mut self) {
        self.brush.clear();
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
                        self.brush.clear();
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
                                let selected = self.phase_tool.is_none()
                                    && self.world.selected_cell_type == kind;
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
                                    self.phase_tool = None;
                                }
                            }
                        });
                    }
                    for pair in [
                        (PhaseTool::Freeze, "Freeze"),
                        (PhaseTool::Melt, "Melt"),
                        (PhaseTool::Vaporize, "Vaporize"),
                        (PhaseTool::Condense, "Condense"),
                    ]
                    .chunks(2)
                    {
                        ui.horizontal(|ui| {
                            let width = (ui.available_width() - ui.spacing().item_spacing.x) / 2.0;
                            for &(tool, name) in pair {
                                let selected = self.phase_tool == Some(tool);
                                if ui
                                    .add_sized(
                                        vec2(width, 30.0),
                                        Button::new(name).sense(Sense::click_and_drag()).stroke(
                                            Stroke::new(
                                                if selected { 2.0 } else { 1.0 },
                                                if selected { theme::ACCENT } else { theme::LINE },
                                            ),
                                        ),
                                    )
                                    .clicked()
                                {
                                    self.phase_tool = Some(tool);
                                }
                            }
                        });
                    }
                    let (_, material_name, material_description, _) = MATERIALS
                        .iter()
                        .find(|(kind, _, _, _)| *kind == self.world.selected_cell_type)
                        .unwrap_or(&MATERIALS[0]);
                    let (name, description) = match self.phase_tool {
                        Some(PhaseTool::Freeze) => (
                            "Freeze",
                            "Turns existing water into ice without changing its quantity.",
                        ),
                        Some(PhaseTool::Melt) => (
                            "Melt",
                            "Returns ice or snow to water. No material is added or removed.",
                        ),
                        Some(PhaseTool::Vaporize) => (
                            "Vaporize",
                            "Turns existing water into Steam without changing its quantity.",
                        ),
                        Some(PhaseTool::Condense) => (
                            "Condense",
                            "Returns Steam to water without adding or removing material.",
                        ),
                        None => (*material_name, *material_description),
                    };
                    egui::Frame::none()
                        .fill(theme::BACKGROUND)
                        .rounding(6.0)
                        .margin(egui::style::Margin::same(10.0))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label(RichText::new(name).strong());
                            ui.label(RichText::new(description).size(12.0).color(theme::MUTED));
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
                    ui.checkbox(&mut self.highlights, "Material highlights").on_hover_text("Static surface accents and fire glow in close views. No decorative animation.");
                    ui.checkbox(&mut self.fast_overview, "Fast overview").on_hover_text(
                        "Smoothed texture below 3 points/cell. Fractional surfaces use detailed geometry.");
                    egui::CollapsingHeader::new("CPU diagnostics").show(ui, |ui| {
                        ui.label(format!("App update: {:.2} ms", self.frame_ms));
                        ui.label(format!("Simulation: {:.2} ms", self.simulation_ms));
                        ui.label(format!("Scene preparation: {:.2} ms", self.render_ms));
                        ui.label(format!("Base scene shapes: {}", self.rendered_shapes));
                        ui.label("Previous frame, CPU only. Excludes GPU and presentation.");
                    });
                    egui::CollapsingHeader::new("Pan view").show(ui, |ui| {
                        let max = self.world.data.ncols() as f32;
                        let x = ui
                            .add(Slider::new(&mut self.world.shift.x, -max..=max).text("Horizontal"))
                            .changed();
                        let y = ui
                            .add(Slider::new(&mut self.world.shift.y, -max..=max).text("Vertical"))
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
                            self.brush.clear();
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
                                self.brush.clear();
                            self.world.rand_generate();
                            }
                            ui.horizontal(|ui| {
                                if ui
                                    .add(Button::new("Smooth").sense(Sense::click_and_drag()))
                                    .clicked()
                                {
                                    self.brush.clear();
                            self.world.smooth();
                                }
                                if ui
                                    .add(Button::new("Clear world").sense(Sense::click_and_drag()))
                                    .clicked()
                                {
                                    self.brush.clear();
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
                        if ui.add_enabled(!self.running, Button::new("Load lab").sense(Sense::click_and_drag()))
                            .on_hover_text("Replaces the current world. Pause first.").clicked() {
                            self.brush.clear(); self.world.load_lab(self.lab); self.fit_view=true;
                        }
                        egui::ComboBox::from_id_source("lab").selected_text(self.lab.name()).show_ui(ui, |ui| {
                            for lab in Lab::ALL {ui.selectable_value(&mut self.lab,lab,lab.name()).on_hover_text(lab.description());}
                        });
                    });
                });
                ui.horizontal(|ui| {
                    ui.label(format!("Brush · {} {}", self.brush.radius * 2 + 1, if self.brush.radius==0 {"cell"} else {"cells"}));
                    ui.add(Slider::new(&mut self.brush.radius, 0..=31).show_value(false));
                    ui.selectable_value(&mut self.brush.square, false, "Round");
                    ui.selectable_value(&mut self.brush.square, true, "Square");
                    if ui.add_enabled(!self.running && self.brush.can_undo(), Button::new("Undo brush").sense(Sense::click_and_drag())).on_hover_text("Paused strokes only · Cmd/Ctrl+Z. Play, Step and world actions clear history.").clicked() {
                        self.undo_brush();
                    }
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
                self.navigate(ctx, &response, rect);
                let owns_pointer =
                    response.dragged_by(egui::PointerButton::Primary) || response.clicked();
                self.brush_input(
                    &ui.input().events,
                    owns_pointer,
                    rect,
                    ui.input().pointer.primary_down(),
                );
                let simulation_start = std::time::Instant::now();
                if self.running {
                    self.world.update();
                }
                self.simulation_ms = simulation_start.elapsed().as_secs_f32() * 1000.0;
                let render_start = std::time::Instant::now();
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
                self.rendered_shapes = 0;
                if self.fast_overview && !self.world.use_shift && self.world.block_size < 3.0 {
                    if let Some((image_rect, image)) = render::overview(&self.world, rect) {
                        if let Some(texture) = &mut self.texture {
                            texture.set(image);
                        } else {
                            self.texture = Some(ctx.load_texture("world", image));
                        }
                        let mut mesh =
                            egui::epaint::Mesh::with_texture(self.texture.as_ref().unwrap().id());
                        mesh.add_rect_with_uv(image_rect, render::UV, Color32::WHITE);
                        painter.add(Shape::mesh(mesh));
                        self.rendered_shapes = 1;
                    }
                } else {
                    // Release the overview allocation when returning to detailed geometry.
                    self.texture = None;
                    self.world.gen_shapes(&mut self.shapes, rect);
                    self.rendered_shapes = self.shapes.len();
                    if self.highlights { render::fire_glow(&self.shapes, &painter, self.world.block_size); }
                    // Solid quads avoid antialias seams between touching cell rectangles.
                    let mut mesh = egui::epaint::Mesh::default();
                    mesh.reserve_vertices(self.shapes.len()*4);
                    mesh.reserve_triangles(self.shapes.len()*2);
                    for shape in &self.shapes {
                        if let Shape::Rect(cell) = shape {
                            mesh.add_colored_rect(cell.rect, theme::scene_color(cell.fill));
                        }
                    }
                    painter.add(Shape::mesh(mesh));
                    let mut accents = 0;
                    for mut shape in self.shapes.drain(..) {
                        if !self.highlights || accents >= 1024 { break; }
                        if let Shape::Rect(cell) = &mut shape { cell.fill = theme::scene_color(cell.fill); }
                        if let Some(accent) = render::surface_accent(&shape, self.world.block_size) {
                            painter.add(accent); accents += 1;
                        }
                    }
                }
                self.render_ms = render_start.elapsed().as_secs_f32() * 1000.0;
                if response.hovered() {
                    if let Some(pos) = response.hover_pos() {
                        let origin = rect.min + self.world.pos_shift;
                        let cell = ((pos - origin) / self.world.block_size).floor();
                        if self.world.bounds_valid(cell) {
                            let radius = self.brush.radius as f32;
                            let center = origin + (cell + vec2(0.5, 0.5)) * self.world.block_size;
                            let extent = (radius + 0.5) * self.world.block_size;
                            if self.brush.square || radius == 0.0 {
                                painter.rect_stroke(
                                    Rect::from_center_size(
                                        center,
                                        vec2(extent * 2.0, extent * 2.0),
                                    ),
                                    0.0,
                                    Stroke::new(1.0, theme::ACCENT),
                                );
                            } else {
                                painter.circle_stroke(
                                    center,
                                    extent,
                                    Stroke::new(1.0, theme::ACCENT),
                                );
                            }
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
        let frame_start = std::time::Instant::now();
        if !ctx.wants_keyboard_input() && ctx.input().key_pressed(egui::Key::Space) {
            self.toggle_running();
        }
        if !ctx.wants_keyboard_input()
            && ctx.input().modifiers.command
            && ctx.input().key_pressed(egui::Key::Z)
        {
            self.undo_brush();
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
                        RichText::new(
                            "Scroll: zoom · Right / middle drag: pan · Cmd/Ctrl+Z: undo brush",
                        )
                        .size(11.0)
                        .color(theme::MUTED),
                    );
                });
            });
        self.toolbox(ctx);
        self.canvas(ctx);
        self.frame_ms = frame_start.elapsed().as_secs_f32() * 1000.0;
        if self.running {
            ctx.request_repaint();
        }
    }
}
