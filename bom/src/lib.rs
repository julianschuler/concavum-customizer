//! The `bom` crate implements the generation of a bill of materials.

use std::fmt::Write;

use config::{Config, KeySize};

/// A bill of materials.
pub struct Bom {
    items: [Item; 12],
}

impl Bom {
    /// Creates a new bill of materials from the given configuration.
    #[must_use]
    pub fn from_config(config: &Config) -> Self {
        let columns = config.finger_cluster.columns.len();
        #[allow(clippy::cast_sign_loss)]
        let rows = i8::from(config.finger_cluster.rows) as usize;
        #[allow(clippy::cast_sign_loss)]
        let thumb_keys = i8::from(config.thumb_cluster.keys) as usize;

        let finger_keys = 2 * columns * rows;
        let thumb_keys = 2 * thumb_keys;
        let total_keys = finger_keys + thumb_keys;

        let (keycaps_1u, keycaps_1_5u) = match config.thumb_cluster.key_size {
            KeySize::U1 => (total_keys, 0),
            KeySize::U1_5 => (finger_keys, thumb_keys),
        };

        let items = [
            Item::new(total_keys, "Key switch Cherry MX compatible", ""),
            Item::new(keycaps_1u, "Keycap DSA 1U", ""),
            Item::new(keycaps_1_5u, "Keycap DSA 1.5U", ""),
            Item::new(1, "TRRS cable 3.5mm 4 pin 50cm", ""),
            Item::new(
                2,
                "TRRS connector 3.5mm 4 pin",
                "https://mouser.com/ProductDetail/490-SJ2-35894CSMT-TR",
            ),
            Item::new(2, "KB2040", "https://mouser.com/ProductDetail/485-5302"),
            Item::new(
                4,
                "FFC connector 1mm pitch 12 pin ",
                "https://mouser.com/ProductDetail/640-FFC3B07-12-T",
            ),
            Item::new(
                2,
                "FFC cable 1mm pitch 12 pin 10cm",
                "https://mouser.com/ProductDetail/640-1012A0101C4084T",
            ),
            Item::new(
                total_keys,
                "1N4148W diode SOD-123",
                "https://mouser.com/ProductDetail/637-1N4148W",
            ),
            Item::new(
                8,
                "M3 countersunk screw 8mm",
                "https://mouser.com/ProductDetail/98-ACC-SC-01 (100 pieces)",
            ),
            Item::new(
                8,
                "M3 heat set insert",
                "https://mouser.com/ProductDetail/761-HM30X226C or \
                https://cnckitchen.store/products/heat-set-insert-m3-x-5-7-100-pieces (100 pieces)",
            ),
            Item::new(
                8,
                "Adhesive rubber feet",
                "https://mouser.com/ProductDetail/144-ASPR-8-062",
            ),
        ];

        Self { items }
    }

    /// Converts the bill of materials to a CSV file.
    #[must_use]
    pub fn to_csv_file(&self) -> String {
        let mut out = "Quantity;Name;URL;\n".to_owned();

        for &Item { amount, name, url } in &self.items {
            if amount > 0 {
                writeln!(out, "{amount};{name};{url};").expect("format should never fail");
            }
        }

        out
    }
}

// An item in a BOM.
struct Item {
    amount: usize,
    name: &'static str,
    url: &'static str,
}

impl Item {
    fn new(amount: usize, name: &'static str, url: &'static str) -> Self {
        Self { amount, name, url }
    }
}
