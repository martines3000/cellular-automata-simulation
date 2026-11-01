use super::*;

impl World {
    pub(in crate::app) fn apply_phase_brush(
        &mut self,
        pointer_pos: Option<Pos2>,
        clip_rect: Rect,
        tool: PhaseTool,
    ) {
        let Some(index) = self.editable_index(pointer_pos, clip_rect) else {
            return;
        };
        let cell = &mut self.data[index];
        match tool {
            PhaseTool::Freeze
                if matches!(cell.cell_type, CellType::Water | CellType::None)
                    && cell.water_quantity() > 0.0 =>
            {
                cell.cell_type = CellType::Ice;
            }
            PhaseTool::Melt => Self::melt_cell(cell),
            PhaseTool::Vaporize
                if matches!(cell.cell_type, CellType::Water | CellType::None)
                    && cell.water_quantity() > 0.0 =>
            {
                cell.cell_type = CellType::Steam;
            }
            PhaseTool::Condense => Self::condense_cell(cell),
            _ => {}
        }
    }

    pub(super) fn melt_cell(cell: &mut Cell) {
        if matches!(cell.cell_type, CellType::Ice | CellType::Snow) {
            cell.cell_type = if cell.water_quantity() >= MIN_MASS {
                CellType::Water
            } else {
                CellType::None
            };
        }
    }

    pub(super) fn update_frozen(&mut self) {
        let (rows, cols) = self.data.dim();
        for y in 1..rows - 1 {
            for x in 1..cols - 1 {
                let cell = self.data[[y, x]];
                if !matches!(cell.cell_type, CellType::Ice | CellType::Snow) {
                    continue;
                }
                if cell.cell_type == CellType::Snow
                    && SMALL_NEIGHBOURHOOD.iter().any(|&(dy, dx)| {
                        let neighbor =
                            self.data[[(y as i32 + dy) as usize, (x as i32 + dx) as usize]];
                        neighbor.cell_type == CellType::Water && neighbor.mass > 0.0
                    })
                {
                    Self::melt_cell(&mut self.tmp[[y, x]]);
                    continue;
                }
                // Destinations were liquid/empty in the snapshot, never another old
                // frozen source: a particle cannot be processed twice this phase.
                if cell.cell_type == CellType::Ice
                    && self.data[[y - 1, x]].cell_type == CellType::Water
                    && self.tmp[[y - 1, x]].cell_type == CellType::Water
                    && self.tmp[[y - 1, x]].mass > 0.0
                {
                    self.tmp.swap([y, x], [y - 1, x]);
                    continue;
                }
                if self.data[[y + 1, x]].cell_type == CellType::None
                    && self.tmp[[y + 1, x]].cell_type == CellType::None
                {
                    self.move_into_empty([y, x], [y + 1, x], cell.cell_type, cell.mass);
                } else if cell.cell_type == CellType::Snow {
                    let open = |nx| {
                        self.data[[y, nx]].cell_type == CellType::None
                            && self.tmp[[y, nx]].cell_type == CellType::None
                            && self.data[[y + 1, nx]].cell_type == CellType::None
                            && self.tmp[[y + 1, nx]].cell_type == CellType::None
                    };
                    let nx = match (open(x - 1), open(x + 1)) {
                        (true, true) => Some(if self.rng.gen_bool(0.5) { x - 1 } else { x + 1 }),
                        (true, false) => Some(x - 1),
                        (false, true) => Some(x + 1),
                        _ => None,
                    };
                    if let Some(nx) = nx {
                        self.move_into_empty([y, x], [y + 1, nx], cell.cell_type, cell.mass);
                    }
                }
            }
        }
        mem::swap(&mut self.data, &mut self.tmp);
    }
}
