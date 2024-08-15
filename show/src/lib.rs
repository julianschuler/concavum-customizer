//! The `show` crate defines a trait for showing configuration widgets.

use std::num::NonZeroU8;

use hex_color::HexColor;
use three_d::egui::{Align, Checkbox, CollapsingHeader, DragValue, Layout, RichText, Ui};

pub use three_d::egui;

/// A color.
pub type Color = HexColor;

/// A trait for showing a configuration widget.
pub trait Show {
    /// The title of the widget.
    const TITLE: &'static str = "";

    /// Shows a widget allowing to modify `self`. Returns true if `self` was changed.
    fn show(&mut self, ui: &mut Ui) -> bool;

    /// Shows a widget with a name and description. Returns true if `self` was changed.
    fn show_with_name_and_description(
        &mut self,
        ui: &mut Ui,
        label: &str,
        description: &str,
    ) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label(label).on_hover_text(description);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                changed = self.show(ui);
            });
        });

        changed
    }

    /// Shows a widget as a collapsable section. Returns true if `self` was changed.
    fn show_section(&mut self, ui: &mut Ui) -> bool {
        let mut changed = false;

        CollapsingHeader::new(RichText::new(Self::TITLE).size(14.0))
            .default_open(true)
            .show(ui, |ui| {
                changed = self.show(ui);
            });

        changed
    }
}

impl Show for Color {
    fn show(&mut self, ui: &mut Ui) -> bool {
        let mut changed = false;

        let Color { r, g, b, a } = *self;
        let mut color = [r, g, b];

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            changed = ui.color_edit_button_srgb(&mut color).changed();
        });

        let [r, g, b] = color;
        *self = Color { r, g, b, a };

        changed
    }
}

impl Show for bool {
    fn show(&mut self, ui: &mut Ui) -> bool {
        ui.add(Checkbox::without_text(self)).changed()
    }
}

impl Show for u8 {
    fn show(&mut self, ui: &mut Ui) -> bool {
        ui.add(DragValue::new(self)).changed()
    }
}

impl Show for NonZeroU8 {
    fn show(&mut self, ui: &mut Ui) -> bool {
        let mut value = u8::from(*self);

        let changed = ui
            .add(DragValue::new(&mut value).clamp_range(1.0..=f64::MAX))
            .changed();

        *self = value.try_into().expect("value should be non-zero");

        changed
    }
}

impl<T> Show for Vec<T> {
    fn show(&mut self, _ui: &mut Ui) -> bool {
        false
    }
}
