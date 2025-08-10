//! The `qmk` crate implements the generation of QMK configuration files.

mod keymap;
mod replace_indented;

use keymap::Keymap;

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

    /// Returns the `keymap.c` file.
    #[must_use]
    pub fn keymap_c(&self) -> String {
        Keymap::new(self.columns, self.rows, self.thumb_keys).to_file()
    }
}
