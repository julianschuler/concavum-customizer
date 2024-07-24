//! The `show` crate defines a trait for showing configuration widgets.

use std::num::NonZeroU8;

use hex_color::HexColor;
use three_d::egui::{Align, Checkbox, CollapsingHeader, DragValue, Layout, RichText, Ui};

pub use three_d::egui;

/// A color.
pub type Color = HexColor;

/// A trait for showing a configuration widget.
pub trait Show {
    /// Shows a widget allowing to modify self.
    fn show(&mut self, ui: &mut Ui);

    /// Shows a widget with a name and description.
    fn show_with_name_and_description(&mut self, ui: &mut Ui, label: &str, description: &str) {
        ui.horizontal(|ui| {
            ui.label(label).on_hover_text(description);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                self.show(ui);
            });
        });
    }
}

/// Creates a collapseable section with a title and the given UI function as content.
pub fn parameters_section(ui: &mut Ui, title: &str, add_contents: impl FnOnce(&mut Ui)) {
    CollapsingHeader::new(RichText::new(title).size(14.0))
        .default_open(true)
        .show(ui, add_contents);
}

impl Show for Color {
    fn show(&mut self, ui: &mut Ui) {
        let Color { r, g, b, a } = *self;
        let mut color = [r, g, b];

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.color_edit_button_srgb(&mut color);
        });

        let [r, g, b] = color;
        *self = Color { r, g, b, a }
    }
}

impl Show for bool {
    fn show(&mut self, ui: &mut Ui) {
        ui.add(Checkbox::without_text(self));
    }
}

impl Show for u8 {
    fn show(&mut self, ui: &mut Ui) {
        ui.add(DragValue::new(self));
    }
}

impl Show for NonZeroU8 {
    fn show(&mut self, ui: &mut Ui) {
        let mut value = u8::from(*self);

        ui.add(DragValue::new(&mut value).clamp_range(1.0..=f64::MAX));

        *self = value.try_into().expect("value should be non-zero");
    }
}

impl<T> Show for Vec<T> {
    fn show(&mut self, _ui: &mut Ui) {}
}
