use egui::Color32;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CellType {
    None,
    Water,
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
    pub x: usize,
    pub y: usize,
    pub cell_type: CellType,
    pub mass: f32,
}

impl Cell {
    pub fn color(&self) -> Color32 {
        match self.cell_type {
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
            x: 0,
            y: 0,
            cell_type: CellType::None,
            mass: 0.0,
        }
    }
}
