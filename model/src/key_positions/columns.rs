use std::ops::{Deref, Mul};

use config::{Column as ConfigColumn, FingerCluster};
use glam::{dvec2, dvec3, DAffine3, DVec2, DVec3, Vec3Swizzles};

use crate::{
    geometry::{Line, Plane},
    key_positions::{CURVATURE_HEIGHT, KEY_CLEARANCE},
    util::{corner_point, SideX, SideY},
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
        let key_distance: DVec2 = config.key_distance.into();

        let inner = config
            .columns
            .iter()
            .enumerate()
            .map(|(i, column)| {
                let (curvature_angle, offset, side_angle, side): (f64, DVec2, f64, _) =
                    match *column {
                        ConfigColumn::Normal {
                            curvature_angle,
                            offset,
                        } => (curvature_angle.into(), offset.into(), 0.0, 0.0),
                        ConfigColumn::Side { side_angle } => {
                            let (side, column) = if i == 0 {
                                (1.0, config.columns.get(1))
                            } else {
                                (-1.0, config.columns.get(config.columns.len() - 2))
                            };

                            if let &ConfigColumn::Normal {
                                curvature_angle,
                                offset,
                            } = column.expect("there has to be at least one normal column")
                            {
                                (
                                    curvature_angle.into(),
                                    offset.into(),
                                    side_angle.into(),
                                    side,
                                )
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
                    (0..config.rows.into())
                        .map(|j| {
                            let y = key_distance.y
                                * f64::from(i16::from(j) - i16::from(config.home_row_index));
                            column_transform * DAffine3::from_translation(dvec3(0.0, y, 0.0))
                        })
                        .collect()
                } else {
                    let keycap_radius = key_distance.y / 2.0 / (curvature_angle / 2.0).tan();
                    let curvature_radius = keycap_radius + CURVATURE_HEIGHT;

                    (0..config.rows.into())
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

    pub fn outline_points(&self) -> Vec<DVec2> {
        let key_clearance = &self.key_clearance;
        let bottom_points = self.windows(2).map(|window| {
            let first_left = window[0].first();
            let first_right = window[1].first();

            Self::circumference_point(first_left, first_right, SideY::Bottom, key_clearance)
        });
        let top_points = self.windows(2).map(|window| {
            let last_left = window[0].last();
            let last_right = window[1].last();

            Self::circumference_point(last_left, last_right, SideY::Top, key_clearance)
        });
        let left_points = self.side_circumference_points(SideX::Left, key_clearance);
        let right_points = self.side_circumference_points(SideX::Right, key_clearance);

        bottom_points
            .chain(right_points)
            .chain(top_points.rev())
            .chain(left_points.into_iter().rev())
            .map(Vec3Swizzles::xy)
            .collect()
    }

    pub fn min_z(&self) -> f64 {
        self.iter()
            .flat_map(|column| column.iter().map(|position| position.translation.z))
            .min_by(f64::total_cmp)
            .unwrap_or_default()
    }

    pub fn max_z(&self) -> f64 {
        self.iter()
            .flat_map(|column| column.iter().map(|position| position.translation.z))
            .max_by(f64::total_cmp)
            .unwrap_or_default()
    }

    fn circumference_point(
        left: &DAffine3,
        right: &DAffine3,
        side_y: SideY,
        key_clearance: &DVec2,
    ) -> DVec3 {
        let left_point = corner_point(left, SideX::Right, side_y, key_clearance);
        let right_point = corner_point(right, SideX::Left, side_y, key_clearance);

        // Get point which is more outward
        #[allow(clippy::float_cmp)]
        if (left_point - right_point).y.signum() == side_y.direction() {
            left_point
        } else {
            right_point
        }
    }

    fn side_circumference_points(&self, side_x: SideX, key_clearance: &DVec2) -> Vec<DVec3> {
        let column = match side_x {
            SideX::Left => self.first(),
            SideX::Right => self.last(),
        };
        let first = column.first();
        let last = column.last();

        let lower_corner = corner_point(first, side_x, SideY::Bottom, key_clearance);
        let upper_corner = corner_point(last, side_x, SideY::Top, key_clearance);

        let mut points = vec![lower_corner];

        points.extend(column.windows(2).filter_map(|window| {
            Self::side_circumference_point(&window[0], &window[1], side_x, key_clearance)
        }));

        points.push(upper_corner);

        points
    }

    fn side_circumference_point(
        bottom: &DAffine3,
        top: &DAffine3,
        side_x: SideX,
        key_clearance: &DVec2,
    ) -> Option<DVec3> {
        let outwards_direction = bottom.x_axis;

        // Get point which is more outward
        let offset = (top.translation - bottom.translation).dot(outwards_direction);
        #[allow(clippy::float_cmp)]
        let offset = if offset.signum() == side_x.direction() {
            offset
        } else {
            0.0
        };
        let point = bottom.translation
            + (offset + side_x.direction() * key_clearance.x) * outwards_direction;

        let line = Line::new(point, bottom.y_axis);
        let plane = Plane::new(top.translation, top.z_axis);

        plane.intersection(&line)
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
