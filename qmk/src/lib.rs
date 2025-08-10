//! The `qmk` crate implements the generation of QMK configuration files.

mod keymap;
mod layout;
mod replace_indented;

use keymap::Keymap;
use layout::layout;
use replace_indented::ReplaceIndented;

/// A QMK configuration.
pub struct Config {
    columns: usize,
    rows: usize,
    thumb_keys: usize,
}

impl Config {
    /// Creates a new QMK configuration from the given matrix parameters.
    #[must_use]
    pub fn new(columns: usize, rows: usize, thumb_keys: usize) -> Self {
        Self {
            columns,
            rows,
            thumb_keys,
        }
    }

    /// Returns the content of the `config.h` file.
    #[must_use]
    pub fn config_h(&self) -> &'static str {
        include_str!("config.h")
    }

    /// Returns the content of the `keyboard.json` file.
    #[must_use]
    pub fn keyboard_json(&self) -> String {
        let column_count = self.columns.max(self.thumb_keys);
        let row_count = self.rows + 1;

        let first_pins = [
            "\"GP8\"", "\"GP7\"", "\"GP6\"", "\"GP5\"", "\"GP4\"", "\"GP27\"",
        ];
        let second_pins = [
            "\"GP9\"", "\"GP10\"", "\"GP19\"", "\"GP20\"", "\"GP18\"", "\"GP26\"",
        ];

        let left_columns = first_pins.iter().take(column_count);
        let left_rows = second_pins.iter().take(row_count);
        let right_columns = second_pins.iter().take(column_count);
        let right_rows = first_pins.iter().rev().take(row_count);
        let layout = layout(self.columns, self.rows, self.thumb_keys);

        include_str!("keyboard.json")
            .replace_indented("$left_columns", left_columns)
            .replace_indented("$left_rows", left_rows)
            .replace_indented("$right_columns", right_columns)
            .replace_indented("$right_rows", right_rows)
            .replace_indented("$layout", layout)
    }

    /// Returns the `keymap.c` file.
    #[must_use]
    pub fn keymap_c(&self) -> String {
        Keymap::new(self.columns, self.rows, self.thumb_keys).to_file()
    }
}
