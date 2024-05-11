use std::ops::{Deref, Mul};

use glam::{dvec2, dvec3, DAffine3, DVec2};

use crate::config::{
    Column as ConfigColumn, FingerCluster, PositiveDVec2, CURVATURE_HEIGHT, KEY_CLEARANCE,
};

pub struct Column {
    keys: Vec<DAffine3>,
    pub column_type: ColumnType,
}

#[derive(Clone)]
pub enum ColumnType {
    Normal,
    Side,
}

impl Column {
    pub fn new(keys: Vec<DAffine3>, column_type: &ConfigColumn) -> Self {
        let column_type = match column_type {
            ConfigColumn::Normal { .. } => ColumnType::Normal,
            ConfigColumn::Side { .. } => ColumnType::Side,
        };

        Self { keys, column_type }
    }

    pub fn first(&self) -> &DAffine3 {
        self.keys.first().expect("there has to be at least one row")
    }

    pub fn last(&self) -> &DAffine3 {
        self.keys.last().expect("there has to be at least one row")
    }
}

impl Deref for Column {
    type Target = [DAffine3];

    fn deref(&self) -> &Self::Target {
        &self.keys
    }
}

impl Mul<&Column> for DAffine3 {
    type Output = Column;

    fn mul(self, column: &Column) -> Self::Output {
        let keys = column.keys.iter().map(|&key| self * key).collect();

        Column {
            keys,
            column_type: column.column_type.clone(),
        }
    }
}

pub struct Columns {
    inner: Vec<Column>,
    pub key_clearance: DVec2,
}

impl Columns {
    pub fn from_config(config: &FingerCluster) -> Self {
        let key_distance: PositiveDVec2 = (&config.key_distance).into();

        let inner = config
            .columns
            .iter()
            .enumerate()
            .map(|(i, column)| {
                let (curvature_angle, offset, side_angle, side) = match column {
                    ConfigColumn::Normal {
                        curvature_angle,
                        offset,
                    } => (curvature_angle, offset, 0.0, 0.0),
                    ConfigColumn::Side { side_angle } => {
                        let (side, column) = if i == 0 {
                            (1.0, config.columns.get(1))
                        } else {
                            (-1.0, config.columns.get(config.columns.len() - 2))
                        };

                        if let ConfigColumn::Normal {
                            curvature_angle,
                            offset,
                        } = column.expect("there has to be at least one normal column")
                        {
                            (curvature_angle, offset, **side_angle, side)
                        } else {
                            panic!("there has to be at least one normal column")
                        }
                    }
                };
                let side_angle = side_angle.to_radians();
                let side_angle_tan = side_angle.tan();

                let (x, z_offset) = if side_angle == 0.0 {
                    #[allow(clippy::cast_precision_loss)]
                    (key_distance.x * i as f64, 0.0)
                } else {
                    let (sin, cos) = (side_angle.sin(), side_angle.cos());
                    let side_radius =
                        key_distance.x / 2.0 / (side_angle / 2.0).tan() + CURVATURE_HEIGHT;

                    #[allow(clippy::cast_precision_loss)]
                    (
                        key_distance.x * (i as f64 + side) - side * side_radius * sin,
                        side_radius * (1.0 - cos),
                    )
                };

                let translation = dvec3(x, offset.x, offset.y + z_offset);
                let column_transform = DAffine3::from_translation(translation)
                    * DAffine3::from_rotation_y(side * side_angle);

                let curvature_angle = curvature_angle.to_radians();
                let keys = if curvature_angle == 0.0 {
                    (0..*config.rows)
                        .map(|j| {
                            let y = key_distance.y
                                * f64::from(i16::from(j) - i16::from(config.home_row_index));
                            column_transform * DAffine3::from_translation(dvec3(0.0, y, 0.0))
                        })
                        .collect()
                } else {
                    let keycap_radius = key_distance.y / 2.0 / (curvature_angle / 2.0).tan();
                    let curvature_radius = keycap_radius + CURVATURE_HEIGHT;

                    (0..*config.rows)
                        .map(|j| {
                            let total_angle = curvature_angle
                                * f64::from(i16::from(j) - i16::from(config.home_row_index));
                            let (sin, rcos) = (total_angle.sin(), 1.0 - total_angle.cos());

                            let x = -side
                                * side_angle_tan
                                * (keycap_radius * rcos
                                    + side_angle.signum() * key_distance.y / 2.0 * sin.abs());
                            let y = curvature_radius * sin;
                            let z = curvature_radius * rcos;

                            column_transform
                                * DAffine3::from_translation(dvec3(x, y, z))
                                * DAffine3::from_rotation_x(total_angle)
                        })
                        .collect()
                };

                Column::new(keys, column)
            })
            .collect();

        let key_clearance = dvec2(
            key_distance.x + KEY_CLEARANCE,
            key_distance.y + KEY_CLEARANCE,
        ) / 2.0;

        Self {
            inner,
            key_clearance,
        }
    }

    pub fn first(&self) -> &Column {
        self.inner
            .first()
            .expect("there has to be at least one column")
    }

    pub fn last(&self) -> &Column {
        self.inner
            .last()
            .expect("there has to be at least one column")
    }
}

impl Deref for Columns {
    type Target = [Column];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Mul<Columns> for DAffine3 {
    type Output = Columns;

    fn mul(self, columns: Columns) -> Columns {
        let inner = columns.iter().map(|column| self * column).collect();

        Columns {
            inner,
            key_clearance: columns.key_clearance,
        }
    }
}
