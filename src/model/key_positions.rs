use std::ops::{Deref, Mul};

use glam::{dvec3, DAffine3, DQuat, DVec2, EulerRot};

use crate::{
    config::{Column as ConfigColumn, Config, FingerCluster, PositiveDVec2, ThumbCluster},
    model::geometry::zvec,
};

const CURVATURE_HEIGHT: f64 = 6.6;

pub struct Column {
    pub column_type: ColumnType,
    entries: Vec<DAffine3>,
}

#[derive(Clone)]
pub enum ColumnType {
    Normal,
    Side,
}

impl Column {
    pub fn new(entries: Vec<DAffine3>, column_type: &ConfigColumn) -> Self {
        let column_type = match column_type {
            ConfigColumn::Normal { .. } => ColumnType::Normal,
            ConfigColumn::Side { .. } => ColumnType::Side,
        };

        Self {
            column_type,
            entries,
        }
    }

    pub fn first(&self) -> &DAffine3 {
        self.entries
            .first()
            .expect("there has to be at least one row")
    }

    pub fn last(&self) -> &DAffine3 {
        self.entries
            .last()
            .expect("there has to be at least one row")
    }
}

impl Mul<&Column> for DAffine3 {
    type Output = Column;

    fn mul(self, column: &Column) -> Self::Output {
        let entries = column.entries.iter().map(|&entry| self * entry).collect();

        Column {
            entries,
            column_type: column.column_type.clone(),
        }
    }
}

impl Deref for Column {
    type Target = [DAffine3];

    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}

pub struct Columns(Vec<Column>);

impl Columns {
    fn from_config(config: &FingerCluster) -> Self {
        let key_distance: PositiveDVec2 = (&config.key_distance).into();

        let columns = config
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
                let entries = if curvature_angle == 0.0 {
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

                Column::new(entries, column)
            })
            .collect();

        Self(columns)
    }

    pub fn first(&self) -> &Column {
        self.0.first().expect("there has to be at least one column")
    }

    pub fn last(&self) -> &Column {
        self.0.last().expect("there has to be at least one column")
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

pub struct ThumbKeys(Vec<DAffine3>);

impl ThumbKeys {
    fn from_config(config: &ThumbCluster) -> Self {
        let curvature_angle = config.curvature_angle.to_radians();
        let cluster_rotation = DQuat::from_euler(
            EulerRot::ZYX,
            config.rotation.z.to_radians(),
            config.rotation.y.to_radians(),
            config.rotation.x.to_radians(),
        );
        let key_transform = DAffine3::from_rotation_translation(cluster_rotation, config.offset);

        let keys = if curvature_angle == 0.0 {
            (0..*config.keys)
                .map(|j| {
                    let x = *config.key_distance
                        * f64::from(i16::from(j) - i16::from(config.resting_key_index));

                    key_transform * DAffine3::from_translation(dvec3(x, 0.0, 0.0))
                })
                .collect()
        } else {
            let curvature_radius =
                (*config.key_distance / 2.0 / (curvature_angle / 2.0).tan()) + CURVATURE_HEIGHT;

            (0..*config.keys)
                .map(|i| {
                    let total_angle = curvature_angle
                        * f64::from(i16::from(i) - i16::from(config.resting_key_index));
                    let (sin, rcos) = (total_angle.sin(), 1.0 - total_angle.cos());
                    let translation = dvec3(curvature_radius * sin, 0.0, curvature_radius * rcos);

                    key_transform
                        * DAffine3::from_translation(translation)
                        * DAffine3::from_rotation_y(-total_angle)
                })
                .collect()
        };

        Self(keys)
    }

    pub fn first(&self) -> &DAffine3 {
        self.0
            .first()
            .expect("there has to be at least one thumb key")
    }

    pub fn last(&self) -> &DAffine3 {
        self.0
            .last()
            .expect("there has to be at least one thumb key")
    }
}

impl Deref for ThumbKeys {
    type Target = [DAffine3];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromIterator<DAffine3> for ThumbKeys {
    fn from_iter<T: IntoIterator<Item = DAffine3>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

pub struct KeyPositions {
    pub columns: Columns,
    pub thumb_keys: ThumbKeys,
}

impl KeyPositions {
    pub fn from_config(config: &Config) -> Self {
        let columns = Columns::from_config(&config.finger_cluster);
        let thumb_keys = ThumbKeys::from_config(&config.thumb_cluster);

        Self {
            columns,
            thumb_keys,
        }
    }

    pub fn tilt(self, tilting_angle: DVec2) -> Self {
        const Z_OFFSET: f64 = 12.0;

        let (tilting_x, tilting_y) = (tilting_angle.x.to_radians(), tilting_angle.y.to_radians());
        let tilting_transform =
            DAffine3::from_rotation_y(tilting_y) * DAffine3::from_rotation_x(tilting_x);

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
            .map(|column| self * column)
            .collect();
        let thumb_keys = key_positions
            .thumb_keys
            .iter()
            .map(|&thumb_key| self * thumb_key)
            .collect();

        KeyPositions {
            columns,
            thumb_keys,
        }
    }
}
