use glam::{dvec2, dvec3, DVec2};
use hex_color::HexColor;
use opencascade::{
    primitives::{Shape, Solid},
    workplane::Workplane,
};

use crate::model::{config::EPSILON, helper::zvec};

use super::{config::Config, Component};

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

impl Into<(Component, Component)> for Key {
    fn into(self) -> (Component, Component) {
        (self.keycap.into(), self.switch.into())
    }
}

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
        const Z_OFFEST: f64 = Switch::TOP_HEIGHT;

        let bottom_length = KEYCAP_PITCH * size_u - (KEYCAP_PITCH - BOTTOM_WIDTH);
        let mid_length = KEYCAP_PITCH * size_u - (KEYCAP_PITCH - MIDDLE_WIDTH);
        let top_length = KEYCAP_PITCH * size_u - (KEYCAP_PITCH - TOP_WIDTH);

        let mut bottom = Workplane::xy()
            .translated(zvec(Z_OFFEST))
            .rect(bottom_length, BOTTOM_WIDTH);
        bottom.fillet(BOTTOM_FILLET);

        let mut middle = Workplane::xy()
            .translated(zvec(Z_OFFEST + HEIGHT / 2.0))
            .rect(MIDDLE_WIDTH, mid_length);
        middle.fillet((TOP_FILLET + BOTTOM_FILLET) / 2.0);

        let mut top = Workplane::xy()
            .translated(zvec(Z_OFFEST + HEIGHT))
            .rect(TOP_WIDTH, top_length);
        top.fillet(TOP_FILLET);

        let mut shape = Solid::loft([&bottom, &middle, &top]).to_shape();

        shape.set_global_translation(zvec(Switch::TOP_HEIGHT));

        Self {
            color: config.colors.keycap,
            shape,
        }
    }
}

impl Into<Component> for KeyCap {
    fn into(self) -> Component {
        Component::new(self.shape, self.color)
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
            .translated(dvec3(0.0, -TOP_OFFSET, Self::TOP_HEIGHT))
            .rect(TOP_SIZE.x, TOP_SIZE.y);

        let shape = Solid::loft([&top_lower_rect, &top_upper_rect]).to_shape();
        let (shape, _) = shape.union(&bottom);

        Self {
            color: config.colors.switch,
            shape,
        }
    }
}

impl Into<Component> for Switch {
    fn into(self) -> Component {
        Component::new(self.shape, self.color)
    }
}
