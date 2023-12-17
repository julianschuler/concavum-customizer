use std::ops::{Deref, Mul};

use glam::{dvec3, DAffine3, DVec2};

use crate::model::{
    config::{ColumnType, FingerCluster, PositiveDVec2},
    geometry::{zvec, Rotate, Translate},
    key::Switch,
};

pub struct Column(Vec<DAffine3>);

impl Column {
    pub fn first(&self) -> &DAffine3 {
        self.0
            .first()
            .expect("there should always be at least one row")
    }

    pub fn last(&self) -> &DAffine3 {
        self.0
            .last()
            .expect("there should always be at least one row")
    }
}

impl Deref for Column {
    type Target = [DAffine3];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromIterator<DAffine3> for Column {
    fn from_iter<T: IntoIterator<Item = DAffine3>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

pub struct Columns(Vec<Column>);

impl Columns {
    pub fn first(&self) -> &Column {
        self.0
            .first()
            .expect("there should always be at least one column")
    }

    pub fn last(&self) -> &Column {
        self.0
            .last()
            .expect("there should always be at least one column")
    }
}

impl Deref for Columns {
    type Target = [Column];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromIterator<Column> for Columns {
    fn from_iter<T: IntoIterator<Item = Column>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

pub struct KeyPositions {
    pub columns: Columns,
}

impl KeyPositions {
    pub fn from_config(config: &FingerCluster) -> Self {
        const CURVATURE_HEIGHT: f64 = Switch::TOP_HEIGHT;

        let key_distance: PositiveDVec2 = (&config.key_distance).into();

        let columns = config
            .columns
            .iter()
            .enumerate()
            .map(|(i, column)| {
                let (side_angle, side, offset) = match column.column_type {
                    ColumnType::Normal { offset } => (0.0, 0.0, offset),
                    ColumnType::Side { side_angle } => {
                        let (side, column) = if i == 0 {
                            (1.0, config.columns.get(1))
                        } else {
                            (-1.0, config.columns.get(config.columns.len() - 2))
                        };

                        let offset = column
                            .map(|column| match column.column_type {
                                ColumnType::Normal { offset } => offset,
                                _ => DVec2::default(),
                            })
                            .unwrap_or(DVec2::default());

                        (*side_angle, side, offset)
                    }
                };
                let side_angle = side_angle.to_radians();
                let side_angle_tan = side_angle.tan();

                let (x, z_offset) = if side_angle == 0.0 {
                    (key_distance.x * i as f64, 0.0)
                } else {
                    let (sin, cos) = (side_angle.sin(), side_angle.cos());
                    let side_radius =
                        key_distance.x / 2.0 / (side_angle / 2.0).tan() + CURVATURE_HEIGHT;

                    (
                        key_distance.x * (i as f64 + side) - side * side_radius * sin,
                        side_radius * (1.0 - cos),
                    )
                };

                let translation = dvec3(x, offset.x, offset.y + z_offset);
                let column_transform =
                    DAffine3::from_rotation_y(side * side_angle).translate(translation);

                let curvature_angle = column.curvature_angle.to_radians();
                if curvature_angle == 0.0 {
                    (0..*config.rows)
                        .map(|j| {
                            let y =
                                key_distance.y * (j as i16 - config.home_row_index as i16) as f64;
                            column_transform * DAffine3::from_translation(dvec3(0.0, y, 0.0))
                        })
                        .collect()
                } else {
                    let keycap_radius = key_distance.y / 2.0 / (curvature_angle / 2.0).tan();
                    let curvature_radius = keycap_radius + CURVATURE_HEIGHT;

                    (0..*config.rows)
                        .map(|j| {
                            let total_angle =
                                curvature_angle * (j as i16 - config.home_row_index as i16) as f64;
                            let (sin, rcos) = (total_angle.sin(), 1.0 - total_angle.cos());

                            let x = -side
                                * side_angle_tan
                                * (keycap_radius * rcos
                                    + side_angle.signum() * key_distance.y / 2.0 * sin.abs());
                            let y = curvature_radius * sin;
                            let z = curvature_radius * rcos;

                            column_transform
                                * DAffine3::from_rotation_x(total_angle).translate(dvec3(x, y, z))
                        })
                        .collect()
                }
            })
            .collect();

        Self { columns }
    }

    pub fn tilt(self, tilting_angle: DVec2) -> Self {
        const Z_OFFSET: f64 = 12.0;

        let (tilting_x, tilting_y) = (tilting_angle.x.to_radians(), tilting_angle.y.to_radians());
        let tilting_transform = DAffine3::from_rotation_x(tilting_x).rotate_y(tilting_y);

        let tilted_positions = tilting_transform * self;

        let z_offset = Z_OFFSET
            - tilted_positions
                .columns
                .iter()
                .flat_map(|column| column.iter().map(|position| position.translation.z))
                .min_by(f64::total_cmp)
                .unwrap_or_default();

        DAffine3::from_translation(zvec(z_offset)) * tilted_positions
    }
}

impl Mul<KeyPositions> for DAffine3 {
    type Output = KeyPositions;

    fn mul(self, key_positions: KeyPositions) -> Self::Output {
        let columns = key_positions
            .columns
            .iter()
            .map(|column| column.iter().map(|position| self * *position).collect())
            .collect();

        KeyPositions { columns }
    }
}
