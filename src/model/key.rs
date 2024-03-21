use glam::{dvec2, dvec3, DVec2};
use hex_color::HexColor;
use opencascade::{
    primitives::{Shape, Solid},
    workplane::Workplane,
};

use crate::model::{
    config::{Config, EPSILON},
    geometry::zvec,
    Component,
};

pub struct Key {
    switch: Switch,
    keycap: KeyCap,
}

impl Key {
    pub fn new(config: &Config, size_u: f64) -> Key {
        let switch = Switch::new(config);
        let keycap = KeyCap::new(config, size_u);

        Self { switch, keycap }
    }
}

impl From<Key> for (Component, Component) {
    fn from(key: Key) -> Self {
        (key.keycap.into(), key.switch.into())
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct KeyCap {
    shape: Shape,
    color: HexColor,
}

impl KeyCap {
    pub fn new(config: &Config, size_u: f64) -> KeyCap {
        const KEYCAP_PITCH: f64 = 19.05;
        const BOTTOM_WIDTH: f64 = 18.2;
        const BOTTOM_FILLET: f64 = 0.5;
        const MIDDLE_WIDTH: f64 = 15.8;
        const TOP_WIDTH: f64 = 12.0;
        const TOP_FILLET: f64 = 2.0;
        const HEIGHT: f64 = 7.0;

        let bottom_length = KEYCAP_PITCH * size_u - (KEYCAP_PITCH - BOTTOM_WIDTH);
        let mid_length = KEYCAP_PITCH * size_u - (KEYCAP_PITCH - MIDDLE_WIDTH);
        let top_length = KEYCAP_PITCH * size_u - (KEYCAP_PITCH - TOP_WIDTH);

        let bottom = Workplane::xy()
            .rect(BOTTOM_WIDTH, bottom_length)
            .fillet(BOTTOM_FILLET);

        let middle = Workplane::xy()
            .translated(zvec(HEIGHT / 2.0))
            .rect(MIDDLE_WIDTH, mid_length)
            .fillet((TOP_FILLET + BOTTOM_FILLET) / 2.0);

        let top = Workplane::xy()
            .translated(zvec(HEIGHT))
            .rect(TOP_WIDTH, top_length)
            .fillet(TOP_FILLET);

        let mut shape: Shape = Solid::loft([&bottom, &middle, &top]).into();

        shape.set_global_translation(zvec(Switch::TOP_HEIGHT));

        Self {
            color: config.colors.keycap,
            shape,
        }
    }
}

impl From<KeyCap> for Component {
    fn from(keycap: KeyCap) -> Self {
        Component::new(todo!(), keycap.color)
    }
}

pub struct Switch {
    shape: Shape,
    color: HexColor,
}

impl Switch {
    pub const TOP_HEIGHT: f64 = 6.6;

    pub fn new(config: &Config) -> Switch {
        const BOTTOM_WIDTH: f64 = 14.0;
        const BOTTOM_HEIGHT: f64 = 5.5;
        const TOP_WIDTH: f64 = 15.6;
        const TOP_SIZE: DVec2 = dvec2(11.0, 10.0);
        const TOP_OFFSET: f64 = 1.5;

        let bottom_rect = Workplane::xy()
            .translated(zvec(-BOTTOM_HEIGHT))
            .rect(BOTTOM_WIDTH, BOTTOM_WIDTH);
        let bottom = bottom_rect.to_face().extrude(zvec(BOTTOM_HEIGHT + EPSILON));

        let top_lower_rect = Workplane::xy().rect(TOP_WIDTH, TOP_WIDTH);
        let top_upper_rect = Workplane::xy()
            .translated(dvec3(0.0, TOP_OFFSET, Self::TOP_HEIGHT))
            .rect(TOP_SIZE.x, TOP_SIZE.y);

        let shape: Shape = Solid::loft([&top_lower_rect, &top_upper_rect]).into();
        let shape = shape.union(&bottom.into()).into();

        Self {
            color: config.colors.switch,
            shape,
        }
    }
}

impl From<Switch> for Component {
    fn from(switch: Switch) -> Self {
        Component::new(todo!(), switch.color)
    }
}
