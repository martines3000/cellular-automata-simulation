use egui::Color32;

pub const ICE_COLOR: Color32 = Color32::from_rgb(151, 218, 235);
pub const SNOW_COLOR: Color32 = Color32::from_rgb(237, 243, 247);

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CellType {
    None,
    Water,
    Ice,
    Snow,
    Dirt,
    Sand,
    Wood,
    FireNormal,
    FireBurn,
    Smoke,
    DarkSmoke,
}

#[derive(PartialEq, Clone, Copy)]
pub struct Cell {
    pub cell_type: CellType,
    // Water-equivalent quantity for None/Water/Ice/Snow; lifetime for smoke.
    pub mass: f32,
}

impl Cell {
    pub fn water_quantity(&self) -> f32 {
        if matches!(
            self.cell_type,
            CellType::None | CellType::Water | CellType::Ice | CellType::Snow
        ) {
            self.mass
        } else {
            0.0
        }
    }

    pub fn color(&self) -> Color32 {
        match self.cell_type {
            CellType::Ice => ICE_COLOR,
            CellType::Snow => SNOW_COLOR,
            CellType::Dirt => Color32::BLACK,
            CellType::Water => Color32::BLUE,
            CellType::Sand => Color32::GOLD,
            CellType::Wood => Color32::BROWN,
            CellType::FireNormal => Color32::LIGHT_RED,
            CellType::FireBurn => Color32::DARK_RED,
            CellType::Smoke => Color32::LIGHT_GRAY,
            CellType::DarkSmoke => Color32::DARK_GRAY,
            _ => Color32::WHITE,
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            cell_type: CellType::None,
            mass: 0.0,
        }
    }
}
