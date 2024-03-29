use fidget::Context;
use glam::{dvec2, dvec3, DVec2, DVec3};
use hex_color::HexColor;

use crate::model::{
    config::{Config, EPSILON},
    geometry::zvec,
    primitives::{Bounds, BoxShape, Csg, Rectangle, Result, Shape, Transforms},
    Component,
};

pub struct Key {
    switch: Switch,
    keycap: KeyCap,
}

impl Key {
    pub fn new(config: &Config, size_u: f64) -> Result<Key> {
        let switch = Switch::new(config)?;
        let keycap = KeyCap::new(config, size_u)?;

        Ok(Self { switch, keycap })
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
    pub fn new(config: &Config, size_u: f64) -> Result<KeyCap> {
        const KEYCAP_PITCH: f64 = 19.05;
        const BOTTOM_WIDTH: f64 = 18.2;
        const BOTTOM_FILLET: f64 = 0.5;
        const TOP_WIDTH: f64 = 12.0;
        const HEIGHT: f64 = 7.0;

        let bottom_length = KEYCAP_PITCH * size_u - (KEYCAP_PITCH - BOTTOM_WIDTH);
        let top_length = KEYCAP_PITCH * size_u - (KEYCAP_PITCH - TOP_WIDTH);
        let scale = dvec2(TOP_WIDTH / BOTTOM_WIDTH, top_length / bottom_length);

        let mut context = Context::new();

        let rectangle = Rectangle::new(dvec2(
            BOTTOM_WIDTH - 2.0 * BOTTOM_FILLET,
            bottom_length - 2.0 * BOTTOM_FILLET,
        ));
        let rounded_rectangle = context.offset(rectangle, BOTTOM_FILLET)?;

        let shape = context.extrude(rounded_rectangle, 0.0, HEIGHT)?;
        let shape = context.taper(shape, scale, HEIGHT)?;
        let root = context.translate(shape, zvec(Switch::TOP_HEIGHT))?;

        let bounds = Bounds::new(bottom_length, zvec(Switch::TOP_HEIGHT));
        let shape = Shape::new(&context, root, bounds)?;

        Ok(Self {
            shape,
            color: config.colors.keycap,
        })
    }
}

impl From<KeyCap> for Component {
    fn from(keycap: KeyCap) -> Self {
        Component::new(keycap.shape, keycap.color)
    }
}

pub struct Switch {
    shape: Shape,
    color: HexColor,
}

impl Switch {
    pub const TOP_HEIGHT: f64 = 6.6;

    pub fn new(config: &Config) -> Result<Switch> {
        const BOTTOM_WIDTH: f64 = 14.0;
        const BOTTOM_HEIGHT: f64 = 5.5;
        const TOP_WIDTH: f64 = 15.6;
        const TOP_SIZE: DVec2 = dvec2(11.0, 10.0);
        const TOP_OFFSET: f64 = 1.5;
        let scale = TOP_SIZE / dvec2(TOP_WIDTH, TOP_WIDTH);

        let mut context = Context::new();

        let bottom = BoxShape::new(dvec3(BOTTOM_WIDTH, BOTTOM_WIDTH, BOTTOM_HEIGHT + EPSILON));
        let bottom = context.translate(bottom, zvec(-(BOTTOM_HEIGHT + EPSILON) / 2.0))?;

        let top = BoxShape::new(dvec3(TOP_WIDTH, TOP_WIDTH, Self::TOP_HEIGHT));
        let top = context.translate(top, zvec(Self::TOP_HEIGHT / 2.0))?;
        let top = context.taper(top, scale, Self::TOP_HEIGHT)?;
        let top = context.shear(top, dvec2(0.0, TOP_OFFSET), Self::TOP_HEIGHT)?;

        let union = context.union(bottom, top)?;

        let bounds = Bounds::new(16.0, DVec3::ZERO);
        let shape = Shape::new(&context, union, bounds)?;

        Ok(Self {
            shape,
            color: config.colors.switch,
        })
    }
}

impl From<Switch> for Component {
    fn from(switch: Switch) -> Self {
        Component::new(switch.shape, switch.color)
    }
}
