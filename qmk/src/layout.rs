use std::fmt::{Display, Formatter, Result};

pub struct Key {
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
                "{{\"matrix\": [{matrix_x}, {matrix_y}], \"x\": {x}, \"y\": {y}, \"height\": {height}}}"
            )
        } else {
            write!(
                f,
                "{{\"matrix\": [{matrix_x}, {matrix_y}], \"x\": {x}, \"y\": {y}}}"
            )
        }
    }
}

/// Returns the layout given the matrix parameters.
#[must_use]
pub fn layout(columns: usize, rows: usize, thumb_keys: usize) -> Vec<Key> {
    const CENTER_PADDING: usize = 3;

    #[allow(clippy::cast_precision_loss)]
    let (row_offset, thumb_key_offset) = if columns >= thumb_keys {
        (0.0, (columns - thumb_keys) as f32 + 0.5)
    } else {
        ((thumb_keys - columns) as f32 - 0.5, 0.0)
    };

    #[allow(clippy::cast_precision_loss)]
    (0..rows)
        .flat_map(|row| {
            (0..columns)
                .map(move |column| {
                    let matrix_position = (rows - row, column);
                    let x = row_offset + column as f32;
                    let y = row as f32;

                    Key::finger(matrix_position, x, y)
                })
                .chain((0..columns).map(move |column| {
                    let matrix_position = (2 * rows + 1 - row, columns - 1 - column);
                    let x = row_offset + (CENTER_PADDING + 1 + columns + column) as f32;
                    let y = row as f32;

                    Key::finger(matrix_position, x, y)
                }))
        })
        .chain((0..thumb_keys).map(move |key| {
            let matrix_position = (0, key);
            let x = thumb_key_offset + key as f32;
            let y = rows as f32;

            Key::thumb(matrix_position, x, y)
        }))
        .chain((0..thumb_keys).map(move |key| {
            let matrix_position = (rows + 1, thumb_keys - 1 - key);
            let x = thumb_key_offset + (CENTER_PADDING + thumb_keys + key) as f32;
            let y = rows as f32;

            Key::thumb(matrix_position, x, y)
        }))
        .collect()
}
