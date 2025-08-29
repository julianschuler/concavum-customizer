use std::ops::{Deref, Mul};

use config::{ColumnConfig, ColumnType as ConfigColumnType, FingerCluster};
use glam::{dvec3, DAffine3, DVec2, DVec3, Vec3Swizzles};

use crate::{
    geometry::{vec_y, Line, Plane},
    key_positions::{CURVATURE_HEIGHT, KEY_CLEARANCE},
    util::{corner_point, side_point, SideX, SideY},
};

/// The positions of a column of finger keys.
pub struct Column {
    keys: Vec<DAffine3>,
    /// The type of the column.
    pub column_type: ColumnType,
}

/// The type of a finger key column.
#[derive(Clone, Copy)]
pub enum ColumnType {
    /// A normal column.
    Normal,
    /// A side column.
    Side,
}

impl From<ConfigColumnType> for ColumnType {
    fn from(column_type: ConfigColumnType) -> Self {
        match column_type {
            ConfigColumnType::Normal => ColumnType::Normal,
            ConfigColumnType::LeftSide | ConfigColumnType::RightSide => ColumnType::Side,
        }
    }
}

impl Column {
    /// Creates a new column given the contained keys and a column type.
    pub fn new(keys: Vec<DAffine3>, column_type: ColumnType) -> Self {
        Self { keys, column_type }
    }

    /// Returns the first finger key of the column.
    pub fn first(&self) -> DAffine3 {
        *self.keys.first().expect("there has to be at least one row")
    }

    /// Returns the last finger key of the column.
    pub fn last(&self) -> DAffine3 {
        *self.keys.last().expect("there has to be at least one row")
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
            column_type: column.column_type,
        }
    }
}

/// The columns and clearances of the finger keys.
pub struct Columns {
    inner: Vec<Column>,
    /// The clearances between neighboring finger keys.
    pub key_clearance: DVec2,
    /// The index of the home row.
    pub home_row_index: i8,
}

impl Columns {
    /// Creates the finger key columns from the finger cluster configuration.
    pub fn from_config(config: &FingerCluster) -> Self {
        let key_distance: DVec2 = config.key_distance.into();
        let column_configs: Vec<ColumnConfig> = (&config.columns).into();
        let first_side = column_configs
            .first()
            .map(|column_config| column_config.column_type.side())
            .unwrap_or_default();
        let home_row_index = config.home_row_index.into();

        let inner = column_configs
            .into_iter()
            .enumerate()
            .map(|(i, column_config)| {
                #[allow(clippy::cast_precision_loss)]
                let index = i as f64 - first_side;
                let ColumnConfig {
                    column_type,
                    curvature_angle,
                    offset,
                    side_angle,
                } = column_config;
                let side = column_type.side();

                let side_angle = side_angle.to_radians();
                let side_angle_tan = side_angle.tan();

                let (x, z_offset) = if side_angle == 0.0 {
                    (key_distance.x * index, 0.0)
                } else {
                    let (sin, cos) = (side_angle.sin(), side_angle.cos());
                    let side_radius =
                        key_distance.x / 2.0 / (side_angle / 2.0).tan() + CURVATURE_HEIGHT;

                    (
                        key_distance.x * (index + side) - side * side_radius * sin,
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
                            let y = key_distance.y * f64::from(j - home_row_index);
                            column_transform * DAffine3::from_translation(vec_y(y))
                        })
                        .collect()
                } else {
                    let keycap_radius = key_distance.y / 2.0 / (curvature_angle / 2.0).tan();
                    let curvature_radius = keycap_radius + CURVATURE_HEIGHT;

                    (0..config.rows.into())
                        .map(|j| {
                            let total_angle = curvature_angle * f64::from(j - home_row_index);
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

                Column::new(keys, column_type.into())
            })
            .collect();

        let key_clearance = (key_distance + DVec2::splat(KEY_CLEARANCE)) / 2.0;

        Self {
            inner,
            key_clearance,
            home_row_index,
        }
    }

    /// Returns the first column.
    pub fn first(&self) -> &Column {
        self.inner
            .first()
            .expect("there has to be at least one column")
    }

    /// Returns the last column.
    pub fn last(&self) -> &Column {
        self.inner
            .last()
            .expect("there has to be at least one column")
    }

    /// Returns the points of the finger cluster outline.
    pub fn outline_points(&self) -> Vec<DVec2> {
        let bottom_points = self.windows(2).map(|window| {
            let first_left = window[0].first();
            let first_right = window[1].first();

            Self::outline_point(first_left, first_right, SideY::Bottom, self.key_clearance)
        });
        let top_points = self.windows(2).map(|window| {
            let last_left = window[0].last();
            let last_right = window[1].last();

            Self::outline_point(last_left, last_right, SideY::Top, self.key_clearance)
        });
        let left_points = self.side_outline_points(SideX::Left);
        let right_points = self.side_outline_points(SideX::Right);

        bottom_points
            .chain(right_points)
            .chain(top_points.rev())
            .chain(left_points.into_iter().rev())
            .map(Vec3Swizzles::xy)
            .collect()
    }

    /// Returns the minimum Z value of all finger key positions.
    pub fn min_z(&self) -> f64 {
        self.iter()
            .flat_map(|column| column.iter().map(|position| position.translation.z))
            .min_by(f64::total_cmp)
            .unwrap_or_default()
    }

    /// Returns the maximum Z value of all finger key positions.
    pub fn max_z(&self) -> f64 {
        self.iter()
            .flat_map(|column| column.iter().map(|position| position.translation.z))
            .max_by(f64::total_cmp)
            .unwrap_or_default()
    }

    /// Returns a point along the top or bottom of the finger cluster outline between two keys.
    fn outline_point(
        left: DAffine3,
        right: DAffine3,
        side_y: SideY,
        key_clearance: DVec2,
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

    /// Returns the points along the left or right side of the finger cluster outline.
    pub fn side_outline_points(&self, side_x: SideX) -> Vec<DVec3> {
        let column = match side_x {
            SideX::Left => self.first(),
            SideX::Right => self.last(),
        };
        let first = column.first();
        let last = column.last();

        let lower_corner = corner_point(first, side_x, SideY::Bottom, self.key_clearance);
        let upper_corner = corner_point(last, side_x, SideY::Top, self.key_clearance);

        let mut points = vec![lower_corner];

        points.extend(column.windows(2).filter_map(|window| {
            Self::side_outline_point(window[0], window[1], side_x, self.key_clearance)
        }));

        points.push(upper_corner);

        points
    }

    /// Returns the point between two keys along the left or right side of the cluster outline.
    fn side_outline_point(
        bottom: DAffine3,
        top: DAffine3,
        side_x: SideX,
        key_clearance: DVec2,
    ) -> Option<DVec3> {
        let outwards_direction = bottom.x_axis;

        let plane = Plane::new(bottom.translation, bottom.x_axis);

        // Get point which is more outward
        let offset = plane.signed_distance_to(top.translation);
        #[allow(clippy::float_cmp)]
        let offset = if offset.signum() == side_x.direction() {
            offset
        } else {
            0.0
        };

        let point = side_point(bottom, side_x.into(), key_clearance) + offset * outwards_direction;

        let line = Line::new(point, bottom.y_axis);
        let plane = Plane::new(
            (bottom.translation + top.translation) / 2.0,
            (bottom.y_axis + top.y_axis) / 2.0,
        );

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
            home_row_index: columns.home_row_index,
        }
    }
}
