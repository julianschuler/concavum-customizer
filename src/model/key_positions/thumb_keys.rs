use std::ops::{Deref, Mul};

use glam::{dvec2, dvec3, DAffine3, DQuat, DVec2, EulerRot};

use crate::config::{ThumbCluster, CURVATURE_HEIGHT, KEY_CLEARANCE};

pub struct ThumbKeys {
    inner: Vec<DAffine3>,
    pub key_clearance: DVec2,
}

impl ThumbKeys {
    pub fn from_config(config: &ThumbCluster) -> Self {
        let key_distance = *config.key_distance;

        let curvature_angle = config.curvature_angle.to_radians();
        let cluster_rotation = DQuat::from_euler(
            EulerRot::ZYX,
            config.rotation.z.to_radians(),
            config.rotation.y.to_radians(),
            config.rotation.x.to_radians(),
        );
        let key_transform = DAffine3::from_rotation_translation(cluster_rotation, config.offset);

        let inner = if curvature_angle == 0.0 {
            (0..*config.keys)
                .map(|j| {
                    let x = key_distance
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

        let key_clearance = dvec2(
            key_distance + KEY_CLEARANCE,
            1.5 * key_distance + KEY_CLEARANCE,
        ) / 2.0;

        Self {
            inner,
            key_clearance,
        }
    }

    pub fn first(&self) -> &DAffine3 {
        self.inner
            .first()
            .expect("there has to be at least one thumb key")
    }

    pub fn last(&self) -> &DAffine3 {
        self.inner
            .last()
            .expect("there has to be at least one thumb key")
    }
}

impl Deref for ThumbKeys {
    type Target = [DAffine3];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Mul<ThumbKeys> for DAffine3 {
    type Output = ThumbKeys;

    fn mul(self, thumb_keys: ThumbKeys) -> ThumbKeys {
        let inner = thumb_keys
            .iter()
            .map(|&thumb_key| self * thumb_key)
            .collect();

        ThumbKeys {
            inner,
            key_clearance: thumb_keys.key_clearance,
        }
    }
}
