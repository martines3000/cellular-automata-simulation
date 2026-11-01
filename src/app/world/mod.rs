pub mod cell;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod edit_state_tests;

#[cfg(test)]
mod wood_tests;

#[cfg(test)]
mod timing_tests;

#[cfg(test)]
mod app_timing_tests;

#[cfg(test)]
mod buffer_tests;

#[cfg(test)]
mod render_tests;

#[cfg(test)]
mod fixtures;

#[cfg(test)]
mod mixed_tests;

#[cfg(test)]
mod benchmarks;

use std::{
    mem,
    time::{Duration, Instant},
};

use egui::{pos2, vec2, Color32, Pos2, Rect, Rounding, Shape, Vec2};
use ndarray::Array2;

use cell::*;
use rand::{rngs::StdRng, Rng, SeedableRng};

const MIN_FLOW: f32 = 0.5;
const MAX_FLOW: f32 = 3.0;
const MAX_COMPRESS: f32 = 0.3;
const MIN_DRAW: f32 = 0.1;
const MIN_MASS: f32 = 0.01;
const MAX_MASS: f32 = 10.0;
const FLOW_SMOOTH: f32 = 0.75;
const MAX_STEPS_PER_UPDATE: usize = 16;
const NANOS_PER_SECOND: u128 = 1_000_000_000;

const NEIGHBOURHOOD: [(i32, i32); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, -1),
    (0, 1),
];

const SMALL_NEIGHBOURHOOD: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

pub struct World {
    pub data: Array2<Cell>,
    pub tmp: Array2<Cell>,
    pub block_size: f32,
    pub pos_shift: Vec2,
    pub shift: Vec2,
    pub steps_per_second: i32,
    pub threshold: f32,
    pub selected_cell_type: CellType,
    pub use_shift: bool,
    step_credit: u128,
    num_of_blocks: usize,
    last_frame_time: Instant,
    rng: StdRng,
}

impl World {
    pub fn new(num_of_blocks: usize) -> Self {
        Self {
            data: Array2::default((num_of_blocks, num_of_blocks)),
            tmp: Array2::default((num_of_blocks, num_of_blocks)),
            block_size: 5.0,
            pos_shift: vec2(0.0, 0.0),
            shift: vec2(0.0, 0.0),
            steps_per_second: 60,
            threshold: 0.5,
            step_credit: 0,
            num_of_blocks,
            last_frame_time: Instant::now(),
            selected_cell_type: CellType::Water,
            rng: StdRng::from_entropy(),
            use_shift: false,
        }
    }

    pub fn gen_shapes(&self, shapes: &mut Vec<Shape>, rect: Rect) {
        if !rect.is_finite()
            || !rect.is_positive()
            || !self.pos_shift.is_finite()
            || !self.block_size.is_finite()
            || self.block_size <= 0.0
        {
            return;
        }
        let (rows, cols) = self.data.dim();
        let start_x = (-self.pos_shift.x / self.block_size)
            .floor()
            .clamp(0.0, cols as f32) as usize;
        let end_x = ((rect.width() - self.pos_shift.x) / self.block_size)
            .ceil()
            .clamp(0.0, cols as f32) as usize;
        // Shifted wood can extend one cell below its nominal row.
        let start_y =
            ((-self.pos_shift.y / self.block_size).floor() - 1.0).clamp(0.0, rows as f32) as usize;
        let end_y = ((rect.height() - self.pos_shift.y) / self.block_size)
            .ceil()
            .clamp(0.0, rows as f32) as usize;
        let origin = rect.min + self.pos_shift;
        let initial_len = shapes.len();
        for y in start_y..end_y {
            for x in start_x..end_x {
                let cell = &self.data[[y, x]];
                let above_water = y > 0 && self.data[[y - 1, x]].cell_type == CellType::Water;
                let mut min_y = y as f32;
                let mut max_y = min_y + 1.0;
                let color = if cell.cell_type == CellType::Water
                    || (cell.cell_type == CellType::None && above_water)
                {
                    if cell.mass > MIN_DRAW {
                        if self.use_shift && !above_water {
                            min_y += (1.0 - cell.mass / MAX_MASS).clamp(0.0, 1.0);
                        }
                        Color32::BLUE
                    } else {
                        continue;
                    }
                } else if cell.cell_type == CellType::Wood
                    && y + 1 < rows
                    && self.data[[y + 1, x]].cell_type == CellType::Water
                {
                    if self.use_shift {
                        let shift = (1.0 - self.data[[y + 1, x]].mass / MAX_MASS).clamp(0.0, 1.0);
                        min_y += shift;
                        max_y += shift;
                    }
                    Color32::BROWN
                } else if cell.cell_type == CellType::None {
                    continue;
                } else {
                    cell.color()
                };
                let cell_rect = Rect::from_min_max(
                    origin + vec2(self.block_size * x as f32, self.block_size * min_y),
                    origin + vec2(self.block_size * (x + 1) as f32, self.block_size * max_y),
                );
                if cell_rect.intersect(rect).is_positive() {
                    let appended = shapes.len() > initial_len;
                    match shapes.last_mut() {
                        Some(Shape::Rect(previous))
                            if appended
                                && previous.fill == color
                                && previous.rect.max.x == cell_rect.min.x
                                && previous.rect.min.y == cell_rect.min.y
                                && previous.rect.max.y == cell_rect.max.y =>
                        {
                            previous.rect.max.x = cell_rect.max.x;
                        }
                        _ => shapes.push(Shape::rect_filled(cell_rect, Rounding::none(), color)),
                    }
                }
            }
        }
    }

    pub fn bounds_valid(&self, block: Vec2) -> bool {
        (block.x as i32) > 0
            && (block.x as usize) < self.num_of_blocks - 1
            && (block.y as i32) > 0
            && (block.y as usize) < self.num_of_blocks - 1
    }

    pub fn add_border(&mut self) {
        let limit_x = self.data.dim().0 - 1;
        let limit_y = self.data.dim().1 - 1;
        for i in 0..self.data.dim().0 {
            self.data[[0, i]].cell_type = CellType::Dirt;
            self.data[[limit_y, i]].cell_type = CellType::Dirt;
        }

        for i in 0..self.data.dim().1 {
            self.data[[i, 0]].cell_type = CellType::Dirt;
            self.data[[i, limit_x]].cell_type = CellType::Dirt;
        }
    }

    pub fn rand_generate(&mut self) {
        for cell in &mut self.data {
            cell.mass = 0.0;
            cell.cell_type = if self.rng.gen::<f32>() < self.threshold {
                CellType::Dirt
            } else {
                CellType::None
            };
        }
        self.add_border();
    }

    pub fn smooth(&mut self) {
        // Editing can run between ticks, when tmp contains an older phase.
        self.tmp.assign(&self.data);
        let mut sum;

        for y in 0..self.tmp.dim().0 as i32 {
            for x in 0..self.tmp.dim().1 as i32 {
                if self.data[[y as usize, x as usize]]
                    .cell_type
                    .ne(&CellType::Dirt)
                    && self.data[[y as usize, x as usize]]
                        .cell_type
                        .ne(&CellType::None)
                {
                    self.tmp[[y as usize, x as usize]].cell_type =
                        self.data[[y as usize, x as usize]].cell_type;
                    continue;
                }

                sum = 0;
                for step in NEIGHBOURHOOD {
                    if y + step.0 < 0
                        || x + step.1 < 0
                        || y + step.0 == self.tmp.dim().0 as i32
                        || x + step.1 == self.tmp.dim().1 as i32
                        || self.data[[(y + step.0) as usize, (x + step.1) as usize]].cell_type
                            == CellType::Dirt
                    {
                        sum += 1;
                    }
                }

                self.tmp[[y as usize, x as usize]].cell_type = if sum > 4 {
                    CellType::Dirt
                } else if sum < 4 {
                    CellType::None
                } else {
                    self.data[[y as usize, x as usize]].cell_type
                };
            }
        }
        mem::swap(&mut self.data, &mut self.tmp);
    }

    fn get_flow(mass: f32, dest_mass: f32) -> f32 {
        let sum = mass + dest_mass;

        if sum <= MAX_MASS {
            MAX_MASS
        } else if sum < 2.0 * MAX_MASS + MAX_COMPRESS {
            (MAX_MASS.powi(2) + sum * MAX_COMPRESS) / (MAX_MASS + MAX_COMPRESS)
        } else {
            (sum + MAX_COMPRESS) / 2.0
        }
    }

    fn move_sand(&mut self, y: usize, x: usize, direction: bool) {
        let nx = if direction { x + 1 } else { x - 1 };
        if self.tmp[[y + 1, nx]].cell_type == CellType::None
            || (self.tmp[[y + 1, nx]].cell_type == CellType::Water && self.move_water(y + 1, nx))
        {
            self.move_into_empty([y, x], [y + 1, nx], CellType::Sand, 0.0);
        }
    }

    fn move_into_empty(&mut self, from: [usize; 2], to: [usize; 2], material: CellType, mass: f32) {
        // Empty destinations may hold passive water; leave it at the vacated source.
        let displaced_water = self.tmp[to].mass;
        self.tmp[from].mass = displaced_water;
        self.tmp[from].cell_type = if displaced_water >= MIN_MASS {
            CellType::Water
        } else {
            CellType::None
        };
        self.tmp[to].cell_type = material;
        self.tmp[to].mass = mass;
    }

    fn move_water(&mut self, y: usize, x: usize) -> bool {
        if self.tmp[[y - 1, x]].cell_type.eq(&CellType::None)
            || self.tmp[[y - 1, x]].cell_type.eq(&CellType::Water)
        {
            self.tmp[[y - 1, x]].cell_type = CellType::Water;
            self.tmp[[y - 1, x]].mass += self.tmp[[y, x]].mass;
            self.tmp[[y, x]].mass = 0.0;
            return true;
        } else if self.tmp[[y, x + 1]].cell_type.eq(&CellType::None)
            || self.tmp[[y, x + 1]].cell_type.eq(&CellType::Water)
        {
            self.tmp[[y, x + 1]].cell_type = CellType::Water;
            self.tmp[[y, x + 1]].mass += self.tmp[[y, x]].mass;
            self.tmp[[y, x]].mass = 0.0;
            return true;
        } else if self.tmp[[y, x - 1]].cell_type.eq(&CellType::None)
            || self.tmp[[y, x - 1]].cell_type.eq(&CellType::Water)
        {
            self.tmp[[y, x - 1]].cell_type = CellType::Water;
            self.tmp[[y, x - 1]].mass += self.tmp[[y, x]].mass;
            self.tmp[[y, x]].mass = 0.0;
            return true;
        } else if self.tmp[[y + 1, x]].cell_type.eq(&CellType::None)
            || self.tmp[[y + 1, x]].cell_type.eq(&CellType::Water)
        {
            self.tmp[[y + 1, x]].cell_type = CellType::Water;
            self.tmp[[y + 1, x]].mass += self.tmp[[y, x]].mass;
            self.tmp[[y, x]].mass = 0.0;
            return true;
        }

        false
    }

    fn wood_stuck(&mut self, y: usize, x: usize) -> (bool, usize) {
        for i in (0..y).rev() {
            if self.tmp[[i, x]].cell_type.eq(&CellType::Wood) {
                continue;
            } else if self.tmp[[i, x]].cell_type.eq(&CellType::None)
                || self.tmp[[i, x]].cell_type.eq(&CellType::Water)
            {
                return (false, i);
            } else {
                break;
            }
        }
        (true, 0)
    }

    fn move_wood(&mut self, from: [usize; 2], to: [usize; 2]) {
        let displaced_mass = self.tmp[to].mass;
        self.tmp[from].mass += displaced_mass;
        self.tmp[from].cell_type = if self.tmp[from].mass >= MIN_MASS {
            CellType::Water
        } else {
            CellType::None
        };
        self.tmp[to].cell_type = CellType::Wood;
        self.tmp[to].mass = 0.0;
    }

    fn flow_water_side(&mut self, y: usize, x: usize, direction: i32, available_mass: f32) -> f32 {
        let nx = (x as i32 + direction) as usize;
        if !matches!(
            self.data[[y, nx]].cell_type,
            CellType::Water | CellType::None | CellType::Wood
        ) {
            return 0.0;
        }
        let mut flow = (available_mass - self.data[[y, nx]].mass) / 3.0;
        if flow > MIN_FLOW {
            flow *= FLOW_SMOOTH;
        }
        flow = flow.clamp(0.0, available_mass);
        if flow <= 0.0 {
            return 0.0;
        }
        if self.tmp[[y, nx]].cell_type == CellType::Wood {
            let beyond = (nx as i32 + direction) as usize;
            if self.tmp[[y, beyond]].cell_type == CellType::None {
                self.move_wood([y, nx], [y, beyond]);
            } else {
                let (stuck, top) = self.wood_stuck(y, nx);
                if stuck {
                    return 0.0;
                }
                self.move_wood([y, nx], [top, nx]);
            }
        }
        self.tmp[[y, x]].mass -= flow;
        self.tmp[[y, nx]].mass += flow;
        if self.tmp[[y, nx]].mass > MIN_MASS {
            self.tmp[[y, nx]].cell_type = CellType::Water;
        }
        flow
    }

    fn update_sand(&mut self) {
        let mut left_diag;
        let mut right_diag;
        let mut left;
        let mut right;

        for y in 0..self.tmp.dim().0 {
            for x in 0..self.tmp.dim().1 {
                if self.data[[y, x]].cell_type.eq(&CellType::Sand) {
                    // Check if sand can move down
                    if self.tmp[[y + 1, x]].cell_type.eq(&CellType::None)
                        || self.tmp[[y + 1, x]].cell_type.eq(&CellType::Water)
                    {
                        self.tmp[[y, x]].cell_type = self.tmp[[y + 1, x]].cell_type;
                        self.tmp[[y, x]].mass = self.tmp[[y + 1, x]].mass;
                        self.tmp[[y + 1, x]].cell_type = CellType::Sand;
                        self.tmp[[y + 1, x]].mass = 0.0;
                    } else {
                        // Check if sand can move diagonally left/right
                        left_diag = self.tmp[[y + 1, x - 1]].cell_type.eq(&CellType::None)
                            || self.tmp[[y + 1, x - 1]].cell_type.eq(&CellType::Water);

                        right_diag = self.tmp[[y + 1, x + 1]].cell_type.eq(&CellType::None)
                            || self.tmp[[y + 1, x + 1]].cell_type.eq(&CellType::Water);

                        left = self.tmp[[y, x - 1]].cell_type.eq(&CellType::Water)
                            || self.tmp[[y, x - 1]].cell_type.eq(&CellType::None);
                        right = self.tmp[[y, x + 1]].cell_type.eq(&CellType::Water)
                            || self.tmp[[y, x + 1]].cell_type.eq(&CellType::None);

                        if left_diag && right_diag && left && right {
                            if self.rng.gen_bool(0.5) {
                                self.move_sand(y, x, true);
                            } else {
                                self.move_sand(y, x, false);
                            }
                        } else if left_diag && left {
                            self.move_sand(y, x, false);
                        } else if right_diag && right {
                            self.move_sand(y, x, true);
                        }
                    }
                }
            }
        }

        mem::swap(&mut self.data, &mut self.tmp);
    }

    fn update_water(&mut self) {
        let mut flow;
        let mut remaining_mass;
        for y in 0..self.tmp.dim().0 {
            for x in 0..self.tmp.dim().1 {
                if self.data[[y, x]].cell_type.eq(&CellType::Water) {
                    remaining_mass = self.data[[y, x]].mass;
                    if remaining_mass < MIN_MASS {
                        // Discard only the old residue; tmp also holds incoming water.
                        self.tmp[[y, x]].mass -= remaining_mass;
                        if self.tmp[[y, x]].mass < MIN_MASS {
                            self.tmp[[y, x]].cell_type = CellType::None;
                        }
                        continue;
                    }

                    if self.data[[y + 1, x]].cell_type.eq(&CellType::Water)
                        || self.data[[y + 1, x]].cell_type.eq(&CellType::None)
                    {
                        flow = World::get_flow(self.data[[y, x]].mass, self.data[[y + 1, x]].mass)
                            - self.data[[y + 1, x]].mass;
                        if flow > MIN_FLOW {
                            flow *= FLOW_SMOOTH;
                        }

                        flow = flow.clamp(0.0, MAX_FLOW.min(remaining_mass));

                        self.tmp[[y, x]].mass -= flow;
                        self.tmp[[y + 1, x]].mass += flow;
                        remaining_mass -= flow;

                        if self.tmp[[y + 1, x]].mass > MIN_MASS {
                            self.tmp[[y + 1, x]].cell_type = CellType::Water;
                        }
                    }

                    if remaining_mass < MIN_FLOW {
                        continue;
                    }

                    if self.data[[y + 1, x]].cell_type.ne(&CellType::Water)
                        || (self.data[[y + 1, x]].cell_type.eq(&CellType::Water)
                            && self.data[[y + 1, x]].mass >= MAX_MASS)
                    {
                        // Both sides use the post-downflow quantity, avoiding a right-first bias.
                        let lateral_mass = remaining_mass;
                        remaining_mass -= self.flow_water_side(y, x, 1, lateral_mass);
                        remaining_mass -= self.flow_water_side(y, x, -1, lateral_mass);
                    }

                    if remaining_mass < MIN_FLOW {
                        continue;
                    }

                    // Pressure
                    if self.data[[y - 1, x]].cell_type.eq(&CellType::None)
                        || self.data[[y - 1, x]].cell_type.eq(&CellType::Water)
                        || self.data[[y - 1, x]].cell_type.eq(&CellType::Wood)
                    {
                        if remaining_mass > (MAX_MASS + MAX_COMPRESS) {
                            if self.tmp[[y - 1, x]].cell_type.eq(&CellType::Wood) {
                                let (stuck, top) = self.wood_stuck(y, x);
                                if !stuck {
                                    self.move_wood([y - 1, x], [top, x]);
                                } else {
                                    continue;
                                }
                            }
                        } else {
                            continue;
                        }

                        flow = remaining_mass
                            - World::get_flow(self.data[[y, x]].mass, self.data[[y - 1, x]].mass);

                        if flow > MIN_FLOW {
                            flow *= 0.8;
                        }

                        flow = flow.clamp(0.0, MAX_FLOW.min(remaining_mass));

                        self.tmp[[y, x]].mass -= flow;
                        self.tmp[[y - 1, x]].mass += flow;

                        if self.tmp[[y - 1, x]].mass >= MIN_MASS {
                            self.tmp[[y - 1, x]].cell_type = CellType::Water;
                        }
                    }
                }
            }
        }
        mem::swap(&mut self.data, &mut self.tmp);
    }

    fn update_wood(&mut self) {
        for y in 0..self.tmp.dim().0 {
            for x in 0..self.tmp.dim().1 {
                if self.data[[y, x]].cell_type.eq(&CellType::Wood)
                    && self.data[[y + 1, x]].cell_type.eq(&CellType::None)
                {
                    self.move_wood([y, x], [y + 1, x]);
                }
            }
        }
        mem::swap(&mut self.data, &mut self.tmp);
    }

    pub fn update_fire(&mut self) {
        for y in 0..self.tmp.dim().0 {
            for x in 0..self.tmp.dim().1 {
                if self.data[[y, x]].cell_type.eq(&CellType::FireNormal)
                    || self.data[[y, x]].cell_type.eq(&CellType::FireBurn)
                {
                    if self.data[[y + 1, x]].cell_type.eq(&CellType::None) {
                        self.move_into_empty([y, x], [y + 1, x], CellType::FireNormal, 0.0);
                    } else {
                        if self.data[[y, x]].cell_type.eq(&CellType::FireBurn) {
                            self.tmp[[y, x]].cell_type = CellType::DarkSmoke;
                            self.tmp[[y, x]].mass = 2.0;
                        } else {
                            self.tmp[[y, x]].cell_type = CellType::Smoke;
                            self.tmp[[y, x]].mass = 1.0;
                        }

                        for step in SMALL_NEIGHBOURHOOD {
                            let neighbor =
                                [(y as i32 + step.0) as usize, (x as i32 + step.1) as usize];
                            if self.tmp[neighbor].cell_type.eq(&CellType::Wood) {
                                self.tmp[neighbor].cell_type = CellType::FireBurn;
                            }
                        }
                    }
                }
            }
        }
        mem::swap(&mut self.data, &mut self.tmp);
    }

    pub fn update_smoke(&mut self) {
        for y in 0..self.tmp.dim().0 {
            for x in 0..self.tmp.dim().1 {
                if self.data[[y, x]].cell_type.eq(&CellType::DarkSmoke)
                    || self.data[[y, x]].cell_type.eq(&CellType::Smoke)
                {
                    if self.data[[y - 1, x]].cell_type.eq(&CellType::None) {
                        let remaining = (self.data[[y, x]].mass - 0.01).max(0.0);
                        let material = if remaining > 0.0 {
                            self.data[[y, x]].cell_type
                        } else {
                            CellType::None
                        };
                        self.move_into_empty([y, x], [y - 1, x], material, remaining);
                    } else {
                        self.tmp[[y, x]].mass = self.data[[y, x]].mass - 0.01;
                        self.tmp[[y, x]].cell_type = self.data[[y, x]].cell_type;
                        if self.tmp[[y, x]].mass <= 0.0 {
                            self.tmp[[y, x]].cell_type = CellType::None;
                            self.tmp[[y, x]].mass = 0.0;
                        }
                    }
                }
            }
        }
        mem::swap(&mut self.data, &mut self.tmp);
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;
        self.advance(elapsed);
    }

    pub fn reset_timing(&mut self) {
        self.step_credit = 0;
        self.last_frame_time = Instant::now();
    }

    fn advance(&mut self, elapsed: Duration) -> usize {
        self.steps_per_second = self.steps_per_second.clamp(1, 360);
        // Nanoseconds × steps/s retains fractional ticks without millisecond truncation.
        let credit = self.step_credit + elapsed.as_nanos() * self.steps_per_second as u128;
        let steps = (credit / NANOS_PER_SECOND).min(MAX_STEPS_PER_UPDATE as u128) as usize;
        // Drop excess whole-tick debt after a stall, keeping its fractional remainder.
        self.step_credit = credit % NANOS_PER_SECOND;
        for _ in 0..steps {
            self.tick();
        }
        steps
    }

    pub(super) fn step_once(&mut self) {
        self.tick();
        self.reset_timing();
    }

    fn tick(&mut self) {
        // Sand
        self.tmp.assign(&self.data);
        self.update_sand();

        // Water
        self.tmp.assign(&self.data);
        self.update_water();

        // Fire
        self.tmp.assign(&self.data);
        self.update_fire();

        self.tmp.assign(&self.data);
        self.update_smoke();

        // Wood
        self.tmp.assign(&self.data);
        self.update_wood();
    }

    pub fn clear(&mut self) {
        for cell in &mut self.data {
            cell.cell_type = CellType::None;
            cell.mass = 0.0;
        }

        self.add_border();
    }

    pub fn resize_and_clear(&mut self, size: usize) {
        assert!(
            size >= 3,
            "a bordered simulation needs at least three cells per side"
        );
        self.data = Array2::default((size, size));
        self.tmp = Array2::default((size, size));
        self.num_of_blocks = size;
        self.add_border();
        self.tmp.assign(&self.data);
        self.shift = vec2(0.0, 0.0);
        self.update_pos();
        self.reset_timing();
    }

    pub fn update_pos(&mut self) {
        self.pos_shift.x = -self.block_size * self.shift.x;
        self.pos_shift.y = -self.block_size * self.shift.y;
    }

    pub fn transform_cell(&mut self, pointer_pos: Option<Pos2>, clip_rect: Rect) {
        if let Some(pos) = pointer_pos {
            if !clip_rect.contains(pos) || pos.x >= clip_rect.max.x || pos.y >= clip_rect.max.y {
                return;
            }
            let block = self.get_block_pos(pos - pos2(clip_rect.left(), clip_rect.top()));

            if self.bounds_valid(block) {
                if self.selected_cell_type.ne(&CellType::Water) {
                    self.data[[block.y as usize, block.x as usize]].cell_type =
                        self.selected_cell_type;
                    self.data[[block.y as usize, block.x as usize]].mass = 0f32;
                } else if self.data[[block.y as usize, block.x as usize]]
                    .cell_type
                    .eq(&CellType::None)
                {
                    self.data[[block.y as usize, block.x as usize]].cell_type =
                        self.selected_cell_type;
                    self.data[[block.y as usize, block.x as usize]].mass = 1f32;
                } else if self.data[[block.y as usize, block.x as usize]]
                    .cell_type
                    .eq(&CellType::Water)
                {
                    self.data[[block.y as usize, block.x as usize]].mass += 2f32;
                }
            }
        }
    }

    fn get_block_pos(&self, pos: Vec2) -> Vec2 {
        ((pos - self.pos_shift) / self.block_size).floor()
    }
}
