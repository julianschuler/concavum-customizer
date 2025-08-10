use std::{
    fmt::Write,
    iter::{once, repeat_n},
};

use itertools::Itertools;

static FULL_SIZE_ROWS: [[&str; 12]; 5] = [
    [
        "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12",
    ],
    [
        "ESC", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "BSPC",
    ],
    [
        "TAB", "Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P", "ENT",
    ],
    [
        "CAPS", "A", "S", "D", "F", "G", "H", "J", "K", "L", "SCLN", "QUOT",
    ],
    [
        "LSFT", "Z", "X", "C", "V", "B", "N", "M", "COMM", "DOT", "SLSH", "RSFT",
    ],
];

static FULL_SIZE_THUMB_KEYS: [&str; 12] = [
    "NO", "NO", "LCTL", "LGUI", "LALT", "SPC", "SPC", "RALT", "RGUI", "RCTL", "NO", "NO",
];

// A keymap consisting of finger and thumb keys.
pub struct Keymap {
    rows: Vec<Row>,
    thumb_keys: Row,
    column_count: usize,
    thumb_key_count: usize,
}

impl Keymap {
    /// Creates a new keymap from the given matrix parameters.
    pub fn new(column_count: usize, row_count: usize, thumb_key_count: usize) -> Self {
        let rows = FULL_SIZE_ROWS
            .iter()
            .skip(FULL_SIZE_ROWS.len() - row_count)
            .map(|full_size_row| Row::from_full_size(full_size_row, column_count))
            .collect();
        let thumb_keys = Row::from_full_size(&FULL_SIZE_THUMB_KEYS, thumb_key_count);

        Self {
            rows,
            thumb_keys,
            column_count,
            thumb_key_count,
        }
    }

    /// Generates the content of the `keymap.c` file.
    pub fn to_file(&self) -> String {
        let comment = self.to_comment();
        let layout_array = self.to_layout_array();

        format!(
            "#include QMK_KEYBOARD_H\n\
            \n\
            const uint16_t PROGMEM keymaps[][MATRIX_ROWS][MATRIX_COLS] = {{\n\
            /*\n{comment} */\n\
            [0] = LAYOUT(\n{layout_array}),\n\
            }};\n"
        )
    }

    /// Returns a comment depicting the keymap.
    fn to_comment(&self) -> String {
        const CENTER_PADDING: usize = 10;

        let (row_padding, thumb_key_padding) = if self.column_count >= self.thumb_key_count {
            (0, 4 * (self.column_count - self.thumb_key_count) + 2)
        } else {
            (4 * (self.thumb_key_count - self.column_count) - 2, 0)
        };

        let mut out = String::new();

        let top_border = comment_top_border(self.column_count);
        let center_border = comment_center_border(self.column_count);

        let row_comments: Vec<_> = self
            .rows
            .iter()
            .map(|row| (row.left.to_comment(), row.right.to_comment()))
            .collect();

        for (left, right) in once((&top_border, &top_border)).chain(Itertools::intersperse(
            row_comments.iter().map(|(left, right)| (left, right)),
            (&center_border, &center_border),
        )) {
            writeln!(
                out,
                " * {:row_padding$}{left}  {:CENTER_PADDING$}  {right}",
                "", ""
            )
            .expect("format should never fail");
        }

        let (left, right) = comment_cluster_borders(self.column_count, self.thumb_key_count);
        writeln!(out, " * {left}{:CENTER_PADDING$}{right}", "").expect("format should never fail");

        let thumb_keys_border = comment_thumb_keys_border(self.thumb_key_count);
        let bottom_border = comment_bottom_border(self.thumb_key_count);

        for (left, right) in [
            (&thumb_keys_border, &thumb_keys_border),
            (
                &self.thumb_keys.left.to_comment(),
                &self.thumb_keys.right.to_comment(),
            ),
            (&bottom_border, &bottom_border),
        ] {
            writeln!(
                out,
                " * {:thumb_key_padding$}{left}{:CENTER_PADDING$}{right}",
                "", ""
            )
            .expect("format should never fail");
        }

        out
    }

    /// Returns the layout array of the keymap.
    fn to_layout_array(&self) -> String {
        const CENTER_PADDING: usize = 10;

        let row_center_padding = CENTER_PADDING + HalfRow::LAYOUT_ARRAY_COLUMN_WIDTH;

        let row_width = HalfRow::LAYOUT_ARRAY_COLUMN_WIDTH * self.column_count;
        let thumb_keys_width = HalfRow::LAYOUT_ARRAY_COLUMN_WIDTH * self.thumb_key_count;

        let (row_padding, thumb_key_padding) = if self.column_count >= self.thumb_key_count {
            (
                0,
                HalfRow::LAYOUT_ARRAY_COLUMN_WIDTH * (self.column_count - self.thumb_key_count)
                    + HalfRow::LAYOUT_ARRAY_COLUMN_WIDTH / 2,
            )
        } else {
            (
                HalfRow::LAYOUT_ARRAY_COLUMN_WIDTH * (self.thumb_key_count - self.column_count)
                    - HalfRow::LAYOUT_ARRAY_COLUMN_WIDTH / 2,
                0,
            )
        };

        let mut out = String::new();

        for row in &self.rows {
            let left = row.left.to_layout_array() + ",";
            let right = row.right.to_layout_array();

            writeln!(
                out,
                "{:row_padding$}{left:<row_width$}{:row_center_padding$}{right},",
                "", ""
            )
            .expect("format should never fail");
        }

        let left = self.thumb_keys.left.to_layout_array() + ",";
        let right = self.thumb_keys.right.to_layout_array();
        writeln!(
            out,
            "{:thumb_key_padding$}{left:<thumb_keys_width$}{:CENTER_PADDING$}{right}",
            "", ""
        )
        .expect("format should never fail");

        out
    }
}

// A row consisting of a left and right half row.
struct Row {
    left: HalfRow,
    right: HalfRow,
}

impl Row {
    /// Creates a new row with the given number of columns from a full size one.
    pub fn from_full_size(default_row: &[&'static str], columns: usize) -> Self {
        let (left, right) = default_row.split_at(default_row.len() / 2);

        let left = HalfRow::new(left.iter().rev().take(columns).rev().copied());
        let right = HalfRow::new(right.iter().take(columns).copied());

        Self { left, right }
    }
}

// A half row consisting of keys.
struct HalfRow {
    keys: Vec<Key>,
}

impl HalfRow {
    /// The width of a column in the layout array
    pub const LAYOUT_ARRAY_COLUMN_WIDTH: usize = 12;

    /// Creates a new half from the given keys.
    pub fn new(keys: impl IntoIterator<Item = impl Into<Key>>) -> Self {
        Self {
            keys: keys.into_iter().map(Into::into).collect(),
        }
    }

    /// Returns the comment representation of the half row.
    pub fn to_comment(&self) -> String {
        let mut out = "│".to_owned();

        for key in &self.keys {
            let comment = key.to_comment();
            write!(out, "{comment:^3}│").expect("format should never fail");
        }

        out
    }

    /// Returns the layout array representation of the half row.
    pub fn to_layout_array(&self) -> String {
        let mut out = String::new();

        for key in &self.keys[..self.keys.len() - 1] {
            let keycode = key.to_keycode() + ",";

            write!(out, "{keycode:<0$}", Self::LAYOUT_ARRAY_COLUMN_WIDTH)
                .expect("format should never fail");
        }
        out += &self
            .keys
            .last()
            .expect("there is always at least one key")
            .to_keycode();

        out
    }
}

// A single key.
struct Key {
    id: &'static str,
}

impl Key {
    // Converts a key to its comment representation.
    pub fn to_comment(&self) -> &'static str {
        match self.id {
            "BSPC" => "BSP",
            "CAPS" => "CPS",
            "SCLN" => ";",
            "QUOT" => "'",
            "COMM" => ",",
            "DOT" => ".",
            "SLSH" => "/",
            "LSFT" | "RSFT" => "SFT",
            "LCTL" | "RCTL" => "CTL",
            "LGUI" | "RGUI" => "GUI",
            "LALT" | "RALT" => "ALT",
            "NO" => "",
            key => key,
        }
    }

    /// Returns the keycode of the key.
    pub fn to_keycode(&self) -> String {
        format!("KC_{id}", id = self.id)
    }
}

impl From<&'static str> for Key {
    fn from(id: &'static str) -> Self {
        Self { id }
    }
}

// Returns the top border of a half row with the given number of keys.
fn comment_top_border(keys: usize) -> String {
    once("┌")
        .chain(repeat_n("┬", keys - 1))
        .chain(once("┐"))
        .join("───")
}

// Returns the center border of a half row with the given number of keys.
fn comment_center_border(keys: usize) -> String {
    once("├")
        .chain(repeat_n("┼", keys - 1))
        .chain(once("┤"))
        .join("───")
}

// Returns the bottom border of a half row with the given number of keys.
fn comment_bottom_border(keys: usize) -> String {
    once("└")
        .chain(repeat_n("┴", keys - 1))
        .chain(once("┘"))
        .join("───")
}

// Returns the center border of a thumb cluster with the given number of keys.
fn comment_thumb_keys_border(keys: usize) -> String {
    repeat_n("│", keys + 1).join("   ")
}

// Returns the left and right borders between finger and thumb clusters.
fn comment_cluster_borders(columns: usize, thumb_keys: usize) -> (String, String) {
    if columns >= thumb_keys {
        let left = once("└")
            .chain(
                repeat_n("─", columns - thumb_keys)
                    .chain(repeat_n("┬", thumb_keys))
                    .interleave(repeat_n("┴", columns)),
            )
            .chain(once("┐"))
            .join("─");

        let right =
            once("┌")
                .chain(repeat_n("┴", columns).interleave(
                    repeat_n("┬", thumb_keys).chain(repeat_n("─", columns - thumb_keys)),
                ))
                .chain(once("┘"))
                .join("─");

        (left, right)
    } else {
        let left = once("┌")
            .chain(
                repeat_n("─", thumb_keys - columns - 1)
                    .chain(repeat_n("┴", columns + 1))
                    .interleave(repeat_n("┬", thumb_keys - 1)),
            )
            .chain(once("┐"))
            .join("─");

        let right = once("┌")
            .chain(
                repeat_n("┴", columns + 1)
                    .chain(repeat_n("─", thumb_keys - columns - 1))
                    .interleave(repeat_n("┬", thumb_keys - 1)),
            )
            .chain(once("┐"))
            .join("─");

        (left, right)
    }
}
