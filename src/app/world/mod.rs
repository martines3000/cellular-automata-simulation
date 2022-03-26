pub mod cell;

use std::{mem, time::Duration};

use egui::{pos2, vec2, Color32, Pos2, Rect, Rounding, Shape, Vec2};
use instant::Instant;
use ndarray::Array2;

use cell::*;
use rand::{prelude::ThreadRng, thread_rng, Rng};

const MIN_FLOW: f32 = 0.5;
const MAX_FLOW: f32 = 3.0;
const MAX_COMPRESS: f32 = 0.3;
const MIN_DRAW: f32 = 0.1;
const MIN_MASS: f32 = 0.01;
const MAX_MASS: f32 = 10.0;
const FLOW_SMOOTH: f32 = 0.75;

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
    pub fps: i32,
    pub threshold: f32,
    pub selected_cell_type: CellType,
    pub use_shift: bool,
    speed: u128,
    num_of_blocks: usize,
    last_frame_time: Instant,
    rng: ThreadRng,
}

impl World {
    pub fn new(num_of_blocks: usize) -> Self {
        let mut _data = Array2::<Cell>::default((num_of_blocks, num_of_blocks));
        let mut _tmp = Array2::<Cell>::default((num_of_blocks, num_of_blocks));

        /*
            Fix cell position
        */
        for y in 0.._data.dim().0 {
            for x in 0.._data.dim().1 {
                _data[[y, x]].x = x;
                _data[[y, x]].y = y;
                _tmp[[y, x]].x = x;
                _tmp[[y, x]].y = y;
            }
        }

        Self {
            data: _data,
            tmp: _tmp,
            block_size: 5.0,
            pos_shift: vec2(0.0, 0.0),
            shift: vec2(0.0, 0.0),
            fps: 60,
            threshold: 0.5,
            speed: World::fps_to_speed(60.0),
            num_of_blocks: num_of_blocks,
            last_frame_time: Instant::now(),
            selected_cell_type: CellType::Water,
            rng: thread_rng(),
            use_shift: false,
        }
    }

    pub fn toggle_shift(&mut self) {
        self.use_shift = !self.use_shift;
    }

    pub fn gen_shapes(&self, shapes: &mut Vec<Shape>, rect: Rect) {
        for y in 0..self.data.dim().0 {
            for x in 0..self.data.dim().1 {
                // Water
                if self.data[[y, x]].cell_type.eq(&CellType::Water)
                    || (y > 0
                        && self.data[[y - 1, x]].cell_type.eq(&CellType::Water)
                        && self.data[[y, x]].cell_type.eq(&CellType::None))
                {
                    if self.data[[y, x]].mass > MIN_DRAW {
                        let shift = if !self.use_shift
                            || self.data[[y - 1, x]].cell_type.eq(&CellType::Water)
                        {
                            0.0
                        } else {
                            (1.0 - (self.data[[y, x]].mass / MAX_MASS)).clamp(0.0, 1.0)
                        };

                        shapes.push(Shape::rect_filled(
                            Rect {
                                min: rect.min
                                    + vec2(
                                        self.block_size * x as f32,
                                        self.block_size * (shift + y as f32),
                                    )
                                    + self.pos_shift,
                                max: rect.min
                                    + vec2(
                                        self.block_size * (x + 1) as f32,
                                        self.block_size * (y + 1) as f32,
                                    )
                                    + self.pos_shift,
                            },
                            Rounding::none(),
                            Color32::BLUE,
                        ));
                    }
                } else if self.data[[y, x]].cell_type.eq(&CellType::Wood)
                    && self.data[[y + 1, x]].cell_type.eq(&CellType::Water)
                {
                    let shift = if self.use_shift {
                        (1.0 - (self.data[[y + 1, x]].mass / MAX_MASS)).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };

                    shapes.push(Shape::rect_filled(
                        Rect {
                            min: rect.min
                                + vec2(
                                    self.block_size * x as f32,
                                    self.block_size * (shift + y as f32),
                                )
                                + self.pos_shift,
                            max: rect.min
                                + vec2(
                                    self.block_size * (x + 1) as f32,
                                    self.block_size * (shift + (y + 1) as f32) as f32,
                                )
                                + self.pos_shift,
                        },
                        Rounding::none(),
                        Color32::BROWN,
                    ));
                } else {
                    shapes.push(Shape::rect_filled(
                        Rect {
                            min: rect.min
                                + vec2(self.block_size * x as f32, self.block_size * y as f32)
                                + self.pos_shift,
                            max: rect.min
                                + vec2(
                                    self.block_size * (x + 1) as f32,
                                    self.block_size * (y + 1) as f32,
                                )
                                + self.pos_shift,
                        },
                        Rounding::none(),
                        self.data[[y, x]].color(),
                    ));
                }
            }
        }
    }

    pub fn fps_to_speed(fps: f32) -> u128 {
        Duration::new(0, (1000000000.0 / fps) as u32).as_millis()
    }

    pub fn bounds_valid(&self, block: Vec2) -> bool {
        if (block.x as i32) > 0
            && (block.x as usize) < self.num_of_blocks - 1
            && (block.y as i32) > 0
            && (block.y as usize) < self.num_of_blocks - 1
        {
            return true;
        }
        return false;
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
        let mut rng = thread_rng();
        for cell in &mut self.data {
            cell.cell_type = if rng.gen::<f32>() < self.threshold {
                CellType::Dirt
            } else {
                CellType::None
            };
        }
        self.add_border();
    }

    pub fn smooth(&mut self) {
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
                    {
                        sum += 1;
                    } else if self.data[[(y + step.0) as usize, (x + step.1) as usize]].cell_type
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
        if direction {
            // Right diagonal
            if self.tmp[[y + 1, x + 1]].cell_type.eq(&CellType::None)
                || (self.tmp[[y + 1, x + 1]].cell_type.eq(&CellType::Water)
                    && self.move_water(y + 1, x + 1))
            {
                self.tmp[[y, x]].cell_type = CellType::None;
                self.tmp[[y + 1, x + 1]].cell_type = CellType::Sand;
            }
        } else {
            // Left diagonal
            if self.tmp[[y + 1, x - 1]].cell_type.eq(&CellType::None)
                || (self.tmp[[y + 1, x - 1]].cell_type.eq(&CellType::Water)
                    && self.move_water(y + 1, x - 1))
            {
                self.tmp[[y, x]].cell_type = CellType::None;
                self.tmp[[y + 1, x - 1]].cell_type = CellType::Sand;
            }
        }
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

    fn lift_wood(&mut self, y: usize, x: usize, top: usize) {
        self.tmp[[y, x]].cell_type = CellType::None;
        self.tmp[[top, x]].cell_type = CellType::Wood;
        self.tmp[[top, x]].mass = 0.0;
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
                        self.tmp[[y, x]].cell_type = CellType::None;
                        self.tmp[[y, x]].mass = 0.0;
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
                        // Right side
                        if self.data[[y, x + 1]].cell_type.eq(&CellType::Water)
                            || self.data[[y, x + 1]].cell_type.eq(&CellType::None)
                            || self.data[[y, x + 1]].cell_type.eq(&CellType::Wood)
                        {
                            flow = (remaining_mass - self.data[[y, x + 1]].mass) / 3.0;

                            if flow > MIN_FLOW {
                                flow *= FLOW_SMOOTH;
                            }

                            flow = flow.clamp(0.0, remaining_mass);

                            if flow >= 0.0 && self.tmp[[y, x + 1]].cell_type.eq(&CellType::Wood) {
                                if self.tmp[[y, x + 2]].cell_type.eq(&CellType::None) {
                                    self.tmp[[y, x + 2]].cell_type = CellType::Wood;
                                } else {
                                    let (stuck, top) = self.wood_stuck(y, x + 1);
                                    if !stuck {
                                        self.lift_wood(y, x + 1, top);
                                    } else {
                                        continue;
                                    }
                                }
                            }

                            self.tmp[[y, x]].mass -= flow;
                            self.tmp[[y, x + 1]].mass += flow;
                            remaining_mass -= flow;

                            if self.tmp[[y, x + 1]].mass > MIN_MASS {
                                self.tmp[[y, x + 1]].cell_type = CellType::Water;
                            }
                        }

                        if remaining_mass < MIN_FLOW {
                            continue;
                        }

                        // Left side
                        if self.data[[y, x - 1]].cell_type.eq(&CellType::Water)
                            || self.data[[y, x - 1]].cell_type.eq(&CellType::None)
                        {
                            flow = (remaining_mass - self.data[[y, x - 1]].mass) / 3.0;
                            if flow > MIN_FLOW {
                                flow *= FLOW_SMOOTH;
                            }

                            flow = flow.clamp(0.0, remaining_mass);

                            if flow > 0.0 && self.tmp[[y, x - 1]].cell_type.eq(&CellType::Wood) {
                                if self.tmp[[y, x - 2]].cell_type.eq(&CellType::None) {
                                    self.tmp[[y, x - 2]].cell_type = CellType::Wood;
                                } else {
                                    let (stuck, top) = self.wood_stuck(y, x - 1);
                                    if !stuck {
                                        self.lift_wood(y, x - 1, top);
                                    } else {
                                        continue;
                                    }
                                }
                            }

                            self.tmp[[y, x]].mass -= flow;
                            self.tmp[[y, x - 1]].mass += flow;
                            remaining_mass -= flow;

                            if self.tmp[[y, x - 1]].mass > MIN_MASS {
                                self.tmp[[y, x - 1]].cell_type = CellType::Water;
                            }
                        }
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
                                    self.lift_wood(y - 1, x, top);
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
                if self.data[[y, x]].cell_type.eq(&CellType::Wood) {
                    if self.data[[y + 1, x]].cell_type.eq(&CellType::None) {
                        self.tmp[[y, x]].cell_type = CellType::None;
                        self.tmp[[y + 1, x]].cell_type = CellType::Wood;
                    }
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
                        self.tmp[[y + 1, x]].cell_type = CellType::FireNormal;
                        self.tmp[[y, x]].cell_type = CellType::None;
                    } else {
                        if self.data[[y, x]].cell_type.eq(&CellType::FireBurn) {
                            self.tmp[[y, x]].cell_type = CellType::DarkSmoke;
                            self.tmp[[y, x]].mass = 2.0;
                        } else {
                            self.tmp[[y, x]].cell_type = CellType::Smoke;
                            self.tmp[[y, x]].mass = 1.0;
                        }

                        for step in SMALL_NEIGHBOURHOOD {
                            if self.tmp[[y + step.0 as usize, x + step.1 as usize]]
                                .cell_type
                                .eq(&CellType::Wood)
                            {
                                self.tmp[[y + step.0 as usize, x + step.1 as usize]].cell_type =
                                    CellType::FireBurn;
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
                        self.tmp[[y - 1, x]].mass = self.data[[y, x]].mass - 0.01;
                        self.tmp[[y - 1, x]].cell_type = self.data[[y, x]].cell_type;
                        self.tmp[[y, x]].cell_type = CellType::None;
                        self.tmp[[y, x]].mass = 0.0;
                        if self.tmp[[y - 1, x]].mass <= 0.0 {
                            self.tmp[[y - 1, x]].cell_type = CellType::None;
                            self.tmp[[y - 1, x]].mass = 0.0;
                        }
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
        let duration_since_last_frame = Instant::now().duration_since(self.last_frame_time);
        if duration_since_last_frame.as_millis().lt(&self.speed) {
            return;
        }

        self.last_frame_time = Instant::now();

        // Sand
        self.tmp = self.data.clone();
        self.update_sand();

        // Water
        self.tmp = self.data.clone();
        self.update_water();

        // Fire
        self.tmp = self.data.clone();
        self.update_fire();

        self.tmp = self.data.clone();
        self.update_smoke();

        // Wood
        self.tmp = self.data.clone();
        self.update_wood();
    }

    pub fn clear(&mut self) {
        for cell in &mut self.data {
            cell.cell_type = CellType::None;
            cell.mass = 0.0;
        }

        self.add_border();
    }

    pub fn update_speed(&mut self) {
        self.speed = World::fps_to_speed(self.fps as f32);
    }

    pub fn update_pos(&mut self) {
        self.pos_shift.x = -self.block_size * self.shift.x;
        self.pos_shift.y = -self.block_size * self.shift.y;
    }

    pub fn transform_cell(&mut self, pointer_pos: Option<Pos2>, clip_rect: Rect) {
        if let Some(pos) = pointer_pos {
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
        return ((pos - self.pos_shift) / self.block_size).floor();
    }
}
