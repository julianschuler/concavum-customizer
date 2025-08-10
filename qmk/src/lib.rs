//! The `qmk` crate implements the generation of QMK configuration files.

mod keyboard;
mod keymap;
mod replace_indented;

use config::Config;

use crate::{keyboard::Keyboard, keymap::Keymap};

/// A set of QMK configuration files.
pub struct Files {
    /// The content of the `config.h` file.
    pub config_h: &'static str,
    /// The content of the `keyboard.json` file.
    pub keyboard_json: String,
    /// The content of the `keymap.c` file.
    pub keymap_c: String,
}

impl Files {
    /// Creates a set of QMK files from the given configuration.
    #[must_use]
    pub fn from_config(config: &Config) -> Self {
        let columns = &config.finger_cluster.columns;
        let columns = usize::from(columns.left_side_column.active)
            + columns.normal_columns.len()
            + usize::from(config.finger_cluster.columns.right_side_column.active);
        #[allow(clippy::cast_sign_loss)]
        let rows = i8::from(config.finger_cluster.rows) as usize;
        #[allow(clippy::cast_sign_loss)]
        let thumb_keys = i8::from(config.thumb_cluster.keys) as usize;

        let config_h = include_str!("config.h");
        let keyboard_json = Keyboard::new(columns, rows, thumb_keys).to_file();
        let keymap_c = Keymap::new(columns, rows, thumb_keys).to_file();

        Self {
            config_h,
            keyboard_json,
            keymap_c,
        }
    }
}
