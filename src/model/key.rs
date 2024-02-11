use hex_color::HexColor;
use libfive::{Region3, Tree};

use crate::model::{
    config::{Colors, EPSILON},
    util::{centered_cubic_region, vec2, vec3},
    Component,
};

pub struct Key {
    switch: Switch,
    keycap: KeyCap,
}

impl Key {
    pub fn new(colors: &Colors, size_u: f32) -> Key {
        let switch = Switch::new(colors.switch);
        let keycap = KeyCap::new(colors.keycap, size_u);

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
    tree: Tree,
    region: Region3,
    color: HexColor,
}

impl KeyCap {
    pub fn new(color: HexColor, size_u: f32) -> KeyCap {
        const KEYCAP_PITCH: f32 = 19.05;
        const BOTTOM_WIDTH: f32 = 18.2;
        const TOP_WIDTH: f32 = 12.0;
        const HEIGHT: f32 = 7.0;

        let bottom_length = KEYCAP_PITCH * size_u - (KEYCAP_PITCH - BOTTOM_WIDTH);
        let top_length = KEYCAP_PITCH * size_u - (KEYCAP_PITCH - TOP_WIDTH);

        let bottom =
            Tree::rectangle_centered_exact(vec2(BOTTOM_WIDTH, bottom_length), vec2(0.0, 0.0));
        let top = Tree::rectangle_centered_exact(vec2(TOP_WIDTH, top_length), vec2(0.0, 0.0));

        let tree =
            bottom
                .loft(top, 0.0.into(), HEIGHT.into())
                .moveit(vec3(0.0, 0.0, Switch::TOP_HEIGHT));

        let region = centered_cubic_region(2.0 * HEIGHT + Switch::TOP_HEIGHT);

        Self {
            tree,
            region,
            color,
        }
    }
}

impl From<KeyCap> for Component {
    fn from(keycap: KeyCap) -> Self {
        Component::new(keycap.tree, keycap.region, keycap.color)
    }
}

pub struct Switch {
    tree: Tree,
    region: Region3,
    color: HexColor,
}

impl Switch {
    pub const TOP_HEIGHT: f32 = 6.6;

    pub fn new(color: HexColor) -> Switch {
        const BOTTOM_WIDTH: f32 = 14.0;
        const BOTTOM_HEIGHT: f32 = 5.5;
        const MIDDLE_WIDTH: f32 = 15.6;
        const TOP_WIDTH: f32 = 10.0;

        let bottom = Tree::box_exact_centered(
            vec3(BOTTOM_WIDTH, BOTTOM_WIDTH, BOTTOM_HEIGHT + EPSILON),
            vec3(0.0, 0.0, (EPSILON - BOTTOM_HEIGHT) / 2.0),
        );

        let top_rect = Tree::rectangle_centered_exact(vec2(TOP_WIDTH, TOP_WIDTH), vec2(0.0, 0.0));
        let top = Tree::rectangle_centered_exact(vec2(MIDDLE_WIDTH, MIDDLE_WIDTH), vec2(0.0, 0.0))
            .loft(top_rect, 0.0.into(), Self::TOP_HEIGHT.into());

        let tree = top.union(bottom);
        let region = centered_cubic_region(BOTTOM_HEIGHT + Self::TOP_HEIGHT);

        Self {
            tree,
            region,
            color,
        }
    }
}

impl From<Switch> for Component {
    fn from(switch: Switch) -> Self {
        Component::new(switch.tree, switch.region, switch.color)
    }
}
