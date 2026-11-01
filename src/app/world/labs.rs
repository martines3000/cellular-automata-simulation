use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::app) enum Lab {
    FloatingIce,
    SteamChamber,
    Snowfall,
}

impl Lab {
    pub(in crate::app) const ALL: [Self; 3] =
        [Self::FloatingIce, Self::SteamChamber, Self::Snowfall];
    pub(in crate::app) fn name(self) -> &'static str {
        match self {
            Self::FloatingIce => "Floating ice",
            Self::SteamChamber => "Steam chamber",
            Self::Snowfall => "Snowfall",
        }
    }
    pub(in crate::app) fn description(self) -> &'static str {
        match self {
            Self::FloatingIce=>"Watch submerged ice and wood rise through the basin. Try Freeze and Melt.",
            Self::SteamChamber=>"Steam rises toward an icy ceiling. Fire starts a short vaporization reaction; there is no heat solver.",
            Self::Snowfall=>"A finite snowfall settles on terraces and melts where it touches water.",
        }
    }
}

impl World {
    pub(in crate::app) fn load_lab(&mut self, lab: Lab) {
        self.clear();
        let n = self.data.nrows();
        // Scale authored features to the interior; tiny test grids remain safe.
        let p = |percent: usize| (percent * n / 100).clamp(1, n - 2);
        let mut fill =
            |xs: std::ops::Range<usize>, ys: std::ops::Range<usize>, kind: CellType, mass: f32| {
                for y in ys {
                    for x in xs.clone() {
                        self.data[[y, x]] = Cell {
                            cell_type: kind,
                            mass,
                        };
                    }
                }
            };
        match lab {
            Lab::FloatingIce => {
                fill(p(10)..p(90), p(85)..p(88), CellType::Dirt, 0.0);
                fill(p(10)..p(13), p(40)..p(85), CellType::Dirt, 0.0);
                fill(p(87)..p(90), p(40)..p(85), CellType::Dirt, 0.0);
                fill(p(13)..p(87), p(55)..p(85), CellType::Water, MAX_MASS);
                fill(p(27)..p(36), p(68)..p(72), CellType::Ice, MAX_MASS);
                fill(p(62)..p(72), p(68)..p(70), CellType::Wood, 0.0);
            }
            Lab::SteamChamber => {
                fill(p(14)..p(86), p(12)..p(15), CellType::Dirt, 0.0);
                fill(p(14)..p(86), p(86)..p(89), CellType::Dirt, 0.0);
                fill(p(14)..p(17), p(15)..p(86), CellType::Dirt, 0.0);
                fill(p(83)..p(86), p(15)..p(86), CellType::Dirt, 0.0);
                fill(p(17)..p(83), p(70)..p(86), CellType::Water, MAX_MASS);
                fill(p(27)..p(40), p(72)..p(82), CellType::Wood, 0.0);
                fill(p(27)..p(40), p(71)..p(72), CellType::FireNormal, 0.0);
                fill(p(60)..p(77), p(15)..p(18), CellType::Ice, MAX_MASS);
                fill(p(62)..p(72), p(48)..p(53), CellType::Steam, 1.0);
            }
            Lab::Snowfall => {
                fill(p(8)..p(43), p(82)..p(85), CellType::Dirt, 0.0);
                fill(p(8)..p(11), p(63)..p(82), CellType::Dirt, 0.0);
                fill(p(40)..p(43), p(63)..p(82), CellType::Dirt, 0.0);
                fill(p(11)..p(40), p(68)..p(82), CellType::Water, MAX_MASS);
                for y in p(8)..p(29) {
                    for x in p(10)..p(90) {
                        if (x * 17 + y * 7) % 13 == 0 {
                            self.data[[y, x]] = Cell {
                                cell_type: CellType::Snow,
                                mass: 1.0,
                            };
                        }
                    }
                }
                for x in p(48)..p(90) {
                    let floor = p(63) + (x - p(48)) * p(16) / (p(90) - p(48)).max(1);
                    for y in floor.min(n - 2)..p(90) {
                        self.data[[y, x]] = Cell {
                            cell_type: CellType::Dirt,
                            mass: 0.0,
                        };
                    }
                }
            }
        }
        self.rng = StdRng::seed_from_u64(0x57a7_a000 + lab as u64);
        self.tmp.assign(&self.data);
        self.reset_timing();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labs_are_repeatable_scaled_and_keep_borders_and_settings() {
        for n in [3, 7, 100, 256, 512, 1024] {
            for lab in Lab::ALL {
                let mut world = World::new(n);
                world.steps_per_second = 30;
                world.selected_cell_type = CellType::Snow;
                world.load_lab(lab);
                let expected = world.data.clone();
                world.data[[1, 1]].mass = 91.0;
                world.load_lab(lab);
                assert!(world.data == expected && world.tmp == expected);
                assert_eq!(world.steps_per_second, 30);
                assert!(world.selected_cell_type == CellType::Snow);
                assert!(world
                    .data
                    .iter()
                    .all(|c| c.mass.is_finite() && c.mass >= 0.0));
                for i in 0..n {
                    for index in [[0, i], [n - 1, i], [i, 0], [i, n - 1]] {
                        assert!(world.data[index].cell_type == CellType::Dirt);
                        assert_eq!(world.data[index].mass, 0.0);
                    }
                }
            }
        }
    }

    #[test]
    fn labs_run_the_conservation_oracle_through_every_phase() {
        for lab in Lab::ALL {
            let mut a = World::new(100);
            a.load_lab(lab);
            let mut b = World::new(100);
            b.load_lab(lab);
            let initial = a.data.clone();
            for _ in 0..80 {
                super::super::mixed_tests::checked_tick(&mut a, lab.name());
                b.tick();
                assert!(a.data == b.data);
            }
            assert!(
                a.data != initial,
                "authored scene must demonstrate movement"
            );
        }
    }
}
