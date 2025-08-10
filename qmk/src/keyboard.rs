use std::fmt::{Display, Formatter, Result};

use crate::replace_indented::ReplaceIndented;

pub struct Keyboard {
    columns: usize,
    rows: usize,
    thumb_keys: usize,
}

impl Keyboard {
    /// Creates a new keyboard from the given matrix parameters.
    pub fn new(columns: usize, rows: usize, thumb_keys: usize) -> Self {
        Self {
            columns,
            rows,
            thumb_keys,
        }
    }

    /// Returns the content the `keyboard.json` file.
    #[must_use]
    pub fn to_file(&self) -> String {
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

        include_str!("keyboard.json")
            .replace_indented("$left_columns", left_columns)
            .replace_indented("$left_rows", left_rows)
            .replace_indented("$right_columns", right_columns)
            .replace_indented("$right_rows", right_rows)
            .replace_indented("$layout", self.layout())
    }

    /// Returns the layout given the matrix parameters.
    #[must_use]
    fn layout(&self) -> Vec<Key> {
        const CENTER_PADDING: usize = 3;

        #[allow(clippy::cast_precision_loss)]
        let (row_offset, thumb_key_offset) = if self.columns >= self.thumb_keys {
            (0.0, (self.columns - self.thumb_keys) as f32 + 0.5)
        } else {
            ((self.thumb_keys - self.columns) as f32 - 0.5, 0.0)
        };

        #[allow(clippy::cast_precision_loss)]
        (0..self.rows)
            .flat_map(|row| {
                (0..self.columns)
                    .map(move |column| {
                        let matrix_position = (self.rows - row, column);
                        let x = row_offset + column as f32;
                        let y = row as f32;

                        Key::finger(matrix_position, x, y)
                    })
                    .chain((0..self.columns).map(move |column| {
                        let matrix_position = (2 * self.rows + 1 - row, self.columns - 1 - column);
                        let x = row_offset + (CENTER_PADDING + 1 + self.columns + column) as f32;
                        let y = row as f32;

                        Key::finger(matrix_position, x, y)
                    }))
            })
            .chain((0..self.thumb_keys).map(move |key| {
                let matrix_position = (0, key);
                let x = thumb_key_offset + key as f32;
                let y = self.rows as f32;

                Key::thumb(matrix_position, x, y)
            }))
            .chain((0..self.thumb_keys).map(move |key| {
                let matrix_position = (self.rows + 1, self.thumb_keys - 1 - key);
                let x = thumb_key_offset + (CENTER_PADDING + self.thumb_keys + key) as f32;
                let y = self.rows as f32;

                Key::thumb(matrix_position, x, y)
            }))
            .collect()
    }
}

// A key in a layout.
struct Key {
    matrix_position: (usize, usize),
    x: f32,
    y: f32,
    height: Option<f32>,
}

impl Key {
    /// Creates a new finger key with the given matrix and key positions.
    pub fn finger(matrix_position: (usize, usize), x: f32, y: f32) -> Self {
        Self {
            matrix_position,
            x,
            y,
            height: None,
        }
    }

    /// Creates a new thumb key with the given matrix and key positions.
    pub fn thumb(matrix_position: (usize, usize), x: f32, y: f32) -> Self {
        Self {
            matrix_position,
            x,
            y,
            height: Some(1.5),
        }
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self {
            matrix_position: (matrix_x, matrix_y),
            x,
            y,
            height,
        } = self;

        if let Some(height) = height {
            write!(
                f,
                "{{\"matrix\": [{matrix_x}, {matrix_y}], \"x\": {x}, \"y\": {y}, \"h\": {height}}}"
            )
        } else {
            write!(
                f,
                "{{\"matrix\": [{matrix_x}, {matrix_y}], \"x\": {x}, \"y\": {y}}}"
            )
        }
    }
}
