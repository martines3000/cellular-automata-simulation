use super::*;

impl World {
    pub(super) fn heat_cell(snapshot: CellType, cell: &mut Cell) {
        match snapshot {
            CellType::Water if cell.cell_type == CellType::Water && cell.mass > 0.0 => {
                cell.cell_type = CellType::Steam;
            }
            CellType::Ice | CellType::Snow => Self::melt_cell(cell),
            _ => {}
        }
    }

    pub(super) fn condense_cell(cell: &mut Cell) {
        if cell.cell_type == CellType::Steam {
            cell.cell_type = if cell.mass >= MIN_MASS {
                CellType::Water
            } else {
                CellType::None
            };
        }
    }

    pub(super) fn update_steam(&mut self) {
        let (rows, cols) = self.data.dim();
        for y in 1..rows - 1 {
            for x in 1..cols - 1 {
                let cell = self.data[[y, x]];
                if cell.cell_type != CellType::Steam {
                    continue;
                }
                if cell.mass == 0.0
                    || SMALL_NEIGHBOURHOOD.iter().any(|&(dy, dx)| {
                        let cold = self.data[[(y as i32 + dy) as usize, (x as i32 + dx) as usize]];
                        matches!(cold.cell_type, CellType::Ice | CellType::Snow) && cold.mass > 0.0
                    })
                {
                    Self::condense_cell(&mut self.tmp[[y, x]]);
                    continue;
                }
                if matches!(
                    self.data[[y - 1, x]].cell_type,
                    CellType::None | CellType::Water
                ) && matches!(
                    self.tmp[[y - 1, x]].cell_type,
                    CellType::None | CellType::Water
                ) {
                    // Swap the whole payload: neither liquid nor passive water is gas lifetime.
                    self.tmp.swap([y, x], [y - 1, x]);
                    continue;
                }
                // Never target an old Steam source, even when another parcel vacated it.
                for ny in [y - 1, y] {
                    let open = |nx, ny| {
                        self.data[[y, nx]].cell_type == CellType::None
                            && self.tmp[[y, nx]].cell_type == CellType::None
                            && self.data[[ny, nx]].cell_type == CellType::None
                            && self.tmp[[ny, nx]].cell_type == CellType::None
                    };
                    let nx = match (open(x - 1, ny), open(x + 1, ny)) {
                        (true, true) => Some(if self.rng.gen_bool(0.5) { x - 1 } else { x + 1 }),
                        (true, false) => Some(x - 1),
                        (false, true) => Some(x + 1),
                        _ => None,
                    };
                    if let Some(nx) = nx {
                        self.move_into_empty([y, x], [ny, nx], CellType::Steam, cell.mass);
                        break;
                    }
                }
            }
        }
        mem::swap(&mut self.data, &mut self.tmp);
    }
}
