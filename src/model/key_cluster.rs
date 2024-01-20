use glam::{dvec2, dvec3, DAffine3, DVec2, DVec3};
use hex_color::HexColor;
use opencascade::primitives::{IntoShape, JoinType, Shape, Solid, Wire};

use crate::model::{
    config::{Config, PositiveDVec2, EPSILON},
    geometry::{zvec, ConvexHull, Line, Plane, Project},
    key_positions::{Column, ColumnType, Columns, KeyPositions},
    Component,
};

pub struct KeyCluster {
    shape: Shape,
    color: HexColor,
    key_positions: KeyPositions,
}

impl KeyCluster {
    pub fn from_config(config: &Config) -> Self {
        let key_positions = KeyPositions::from_config(config).tilt(config.keyboard.tilting_angle);
        let mount = Mount::from_positions(&key_positions, *config.keyboard.circumference_distance);

        let clearances = ClearanceBuilder::new(config, &key_positions, &mount.size).build();

        let mut shape = mount.into_shape();
        for clearance in clearances {
            shape = shape.subtract(&clearance).into();
        }

        Self {
            shape,
            color: config.colors.keyboard,
            key_positions,
        }
    }

    pub fn finger_key_positions(&self) -> Vec<DAffine3> {
        self.key_positions
            .columns
            .iter()
            .flat_map(|column| column.iter())
            .copied()
            .collect()
    }

    pub fn thumb_key_positions(&self) -> Vec<DAffine3> {
        self.key_positions.thumb_keys.to_owned()
    }
}

struct ClearanceBuilder<'a> {
    key_positions: &'a KeyPositions,
    key_clearance: DVec2,
    support_planes: SupportPlanes,
    mount_size: MountSize,
}

impl<'a> ClearanceBuilder<'a> {
    fn new(config: &Config, key_positions: &'a KeyPositions, mount_size: &MountSize) -> Self {
        const KEY_CLEARANCE: f64 = 1.0;

        let key_distance: PositiveDVec2 = (&config.finger_cluster.key_distance).into();
        let key_clearance = dvec2(
            key_distance.x + KEY_CLEARANCE,
            key_distance.y + KEY_CLEARANCE,
        );

        let support_planes = SupportPlanes::from_columns(&key_positions.columns);
        let mount_size = mount_size.to_owned();

        Self {
            key_positions,
            key_clearance,
            support_planes,
            mount_size,
        }
    }

    fn build(self) -> Vec<Shape> {
        let columns = &self.key_positions.columns;
        let first = columns.first();
        let last = columns.last();

        let neighbor = match first.column_type {
            ColumnType::Normal => None,
            ColumnType::Side => columns.get(1),
        };
        let left_clearance = self.side_column_clearance(first, neighbor, false);

        let neighbor = match last.column_type {
            ColumnType::Normal => None,
            ColumnType::Side => columns.get(columns.len() - 2),
        };
        let right_clearance = self.side_column_clearance(last, neighbor, true);

        let mut clearances = vec![left_clearance, right_clearance];

        if let Some(columns) = columns.get(1..columns.len() - 1) {
            clearances.extend(
                columns
                    .iter()
                    .map(|column| self.normal_column_clearance(column).into()),
            );
        }

        clearances
    }

    fn normal_column_clearance(&self, column: &Column) -> Solid {
        let points = self.clearance_points(column);
        let first = column.first();
        let normal = first.x_axis;
        let plane = Plane::new(
            first.translation - self.key_clearance.x / 2.0 * normal,
            normal,
        );

        project_points_to_plane_and_extrude(points, plane, self.key_clearance.x)
    }

    fn side_column_clearance(
        &self,
        column: &Column,
        neighbor: Option<&Column>,
        is_right: bool,
    ) -> Shape {
        let clearance_x = self.key_clearance.x;
        let sign = if is_right { 1.0 } else { -1.0 };
        let column_offset = 2.0 * clearance_x;
        let neighbor_offset = clearance_x / 2.0;
        let extrusion_height = 4.0 * clearance_x;

        // Column clearance
        let first = column.first();
        let normal = first.x_axis;
        let plane = Plane::new(first.translation + sign * column_offset * normal, normal);
        let points = self.clearance_points(column);
        let column_clearance =
            project_points_to_plane_and_extrude(points, plane, -sign * extrusion_height);

        // Combined column and neighbor clearance
        let combined_clearance = if let Some(neighbor) = neighbor {
            let first = neighbor.first();
            let normal = first.x_axis;
            let plane = Plane::new(first.translation - sign * neighbor_offset * normal, normal);
            let points = self.clearance_points(neighbor);
            let neighbor_clearance =
                project_points_to_plane_and_extrude(points, plane, sign * extrusion_height);

            column_clearance.intersect(&neighbor_clearance).into_shape()
        } else {
            column_clearance.into_shape()
        };

        // Combined column, side and neighbor clearance
        let side_clearance = self.side_clearance(is_right);
        if (column.first().x_axis.z <= 0.0) == is_right {
            combined_clearance.intersect(&side_clearance).into()
        } else {
            combined_clearance.union(&side_clearance).into()
        }
    }

    fn side_clearance(&self, is_right: bool) -> Shape {
        let columns = &self.key_positions.columns;
        let column = if is_right {
            columns.last()
        } else {
            columns.first()
        };
        let first = column.first();
        let last = column.last();
        let sign = if is_right { 1.0 } else { -1.0 };

        let lower_corner = first.translation + sign * Mount::PLATE_X_2 * first.x_axis
            - Mount::PLATE_Y_2 * first.y_axis;
        let upper_corner = last.translation
            + sign * Mount::PLATE_X_2 * last.x_axis
            + Mount::PLATE_Y_2 * last.y_axis;
        let outwards_bottom_point = lower_corner - self.mount_size.width * DVec3::Y;
        let outwards_top_point = upper_corner + self.mount_size.width * DVec3::Y;
        let upwards_bottom_point = outwards_bottom_point + 2.0 * self.mount_size.height * DVec3::Z;
        let upwards_top_point = outwards_top_point + 2.0 * self.mount_size.height * DVec3::Z;

        let points = column
            .windows(2)
            .filter_map(|window| Self::side_point(&window[0], &window[1], sign))
            .chain([
                upper_corner,
                outwards_top_point,
                upwards_top_point,
                upwards_bottom_point,
                outwards_bottom_point,
                lower_corner,
            ]);

        let plane = Plane::new(self.mount_size.length * DVec3::NEG_X, DVec3::X);
        project_points_to_plane_and_extrude(points, plane, 3.0 * self.mount_size.length).into()
    }

    fn side_point(bottom: &DAffine3, top: &DAffine3, sign: f64) -> Option<DVec3> {
        let outwards_direction = bottom.x_axis;

        // Get point which is more outward
        let offset = (top.translation - bottom.translation).dot(outwards_direction);
        let offset = if offset.signum() == sign { offset } else { 0.0 };
        let point = bottom.translation + (offset + sign * Mount::PLATE_X_2) * outwards_direction;

        let line = Line::new(point, bottom.y_axis);
        let plane = Plane::new(top.translation, top.z_axis);

        plane.intersection(&line)
    }

    fn clearance_points(&self, column: &Column) -> Vec<DVec3> {
        let first = column.first();
        let last = column.last();

        // All points in the center, if any
        let mut points: Vec<_> = column
            .windows(2)
            .filter_map(|window| {
                let position = window[0];
                let next_position = window[1];
                let line = Line::new(position.translation, position.y_axis);
                let plane = Plane::new(next_position.translation, next_position.z_axis);

                plane.intersection(&line)
            })
            .collect();

        // Upper and lower support points derived from the first and last entries
        let mut lower_support_points = self.support_planes.calculate_support_points(
            first,
            false,
            &column.column_type,
            &self.mount_size,
        );
        lower_support_points.reverse();
        let upper_support_points = self.support_planes.calculate_support_points(
            last,
            true,
            &column.column_type,
            &self.mount_size,
        );

        // Combine upper and lower support points with clearance points to polygon points
        points.extend(upper_support_points);
        points.extend(lower_support_points);

        points
    }
}

impl From<KeyCluster> for Component {
    fn from(cluster: KeyCluster) -> Self {
        Component::new(cluster.shape, cluster.color)
    }
}

struct SupportPlanes {
    lower_plane: Plane,
    upper_plane: Plane,
}

impl SupportPlanes {
    fn from_columns(columns: &Columns) -> Self {
        let reference_column = columns.get(1).unwrap_or_else(|| columns.first());
        let x_axis = reference_column.first().x_axis;
        let normal = x_axis.cross(DVec3::Y);

        let mut lower_points: Vec<_> = columns
            .iter()
            .filter_map(|column| match column.column_type {
                ColumnType::Normal => {
                    let position = column.first();
                    Some(position.translation - Mount::PLATE_Y_2 * position.y_axis)
                }
                ColumnType::Side => None,
            })
            .collect();

        let mut upper_points: Vec<_> = columns
            .iter()
            .filter_map(|column| match column.column_type {
                ColumnType::Normal => {
                    let position = column.last();
                    Some(position.translation + Mount::PLATE_Y_2 * position.y_axis)
                }
                ColumnType::Side => None,
            })
            .collect();

        let lower_plane = Self::calculate_median_plane(normal, &mut lower_points);
        let upper_plane = Self::calculate_median_plane(normal, &mut upper_points);

        Self {
            lower_plane,
            upper_plane,
        }
    }

    fn calculate_median_plane(normal: DVec3, points: &mut Vec<DVec3>) -> Plane {
        points.sort_unstable_by(|position1, position2| {
            normal.dot(*position1).total_cmp(&normal.dot(*position2))
        });

        let median_point = points[points.len() / 2];

        Plane::new(median_point, normal)
    }

    fn calculate_support_points(
        &self,
        position: &DAffine3,
        upper: bool,
        column_type: &ColumnType,
        mount_size: &MountSize,
    ) -> Vec<DVec3> {
        const ALLOWED_DEVIATION: f64 = 1.0;

        let (sign, plane) = if upper {
            (1.0, &self.upper_plane)
        } else {
            (-1.0, &self.lower_plane)
        };
        let point_direction = sign * position.y_axis;
        let point = position.translation + Mount::PLATE_Y_2 * point_direction;

        let point_is_above = plane.signed_distance_to(point) > 0.0;
        let point_direction_is_upwards = point_direction.dot(plane.normal()) > 0.0;

        let projected_point = match column_type {
            ColumnType::Normal => {
                let default = if point_is_above == point_direction_is_upwards {
                    let line = Line::new(point, position.z_axis);

                    plane.intersection(&line).unwrap()
                } else {
                    point.project_to(plane)
                };

                let line = Line::new(point, position.y_axis);
                plane
                    .intersection(&line)
                    .map(|point| {
                        if point.abs_diff_eq(default, ALLOWED_DEVIATION) {
                            point
                        } else {
                            default
                        }
                    })
                    .unwrap_or(default)
            }
            ColumnType::Side => point,
        };

        let mut points = vec![point];
        if !point.abs_diff_eq(projected_point, EPSILON) {
            points.push(projected_point);
        }

        let outwards_point = projected_point + sign * mount_size.width * DVec3::Y;
        let upwards_point = outwards_point + 2.0 * mount_size.height * DVec3::Z;
        points.extend([outwards_point, upwards_point]);
        points
    }
}

#[derive(Clone)]
struct MountSize {
    height: f64,
    width: f64,
    length: f64,
}

struct Mount {
    shape: Solid,
    size: MountSize,
}

impl Mount {
    const PLATE_SIZE: DVec2 = dvec2(17.0, 18.0);
    const PLATE_X_2: f64 = Self::PLATE_SIZE.x / 2.0;
    const PLATE_Y_2: f64 = Self::PLATE_SIZE.y / 2.0;

    fn from_positions(key_positions: &KeyPositions, circumference_distance: f64) -> Self {
        let points: Vec<_> = key_positions
            .columns
            .iter()
            .flat_map(|column| column.iter())
            .chain(key_positions.thumb_keys.iter())
            .flat_map(|position| {
                let x_offset = Self::PLATE_X_2 * position.x_axis;
                let y_offset = Self::PLATE_Y_2 * position.y_axis;

                [
                    position.translation + x_offset + y_offset,
                    position.translation + x_offset - y_offset,
                    position.translation - x_offset + y_offset,
                    position.translation - x_offset - y_offset,
                ]
            })
            .map(|position| dvec2(position.x, position.y))
            .collect();

        let points = ConvexHull::from_points(points);

        let size = Self::calculate_size(key_positions, &points);
        let wire =
            Wire::from_ordered_points(points.iter().map(|point| dvec3(point.x, point.y, 0.0)))
                .expect("wire is created from more than 2 points");
        let wire = wire.offset(circumference_distance, JoinType::Arc);
        let shape = wire.to_face().extrude(zvec(size.height));

        Self { shape, size }
    }

    fn calculate_size(key_positions: &KeyPositions, points: &[DVec2]) -> MountSize {
        let height = key_positions
            .columns
            .iter()
            .flat_map(|column| column.iter().map(|position| position.translation.z))
            .max_by(f64::total_cmp)
            .unwrap_or_default()
            + 20.0;

        let min_y = points
            .iter()
            .map(|point| point.y)
            .min_by(f64::total_cmp)
            .unwrap_or_default();
        let max_y = points
            .iter()
            .map(|point| point.y)
            .max_by(f64::total_cmp)
            .unwrap_or_default();
        let width = max_y - min_y;

        let min_x = points
            .iter()
            .map(|point| point.x)
            .min_by(f64::total_cmp)
            .unwrap_or_default();
        let max_x = points
            .iter()
            .map(|point| point.x)
            .max_by(f64::total_cmp)
            .unwrap_or_default();
        let length = max_x - min_x;

        MountSize {
            height,
            width,
            length,
        }
    }
}

impl IntoShape for Mount {
    fn into_shape(self) -> Shape {
        self.shape.into_shape()
    }
}

fn project_points_to_plane_and_extrude(
    points: impl IntoIterator<Item = DVec3>,
    plane: Plane,
    height: f64,
) -> Solid {
    let points = points.into_iter().map(|point| point.project_to(&plane));
    let wire = Wire::from_ordered_points(points).expect("wire is created from more than 2 points");

    wire.to_face().extrude(height * plane.normal())
}
