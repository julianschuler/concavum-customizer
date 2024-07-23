use std::num::NonZeroU8;

use config::{
    Color, Colors, Columns, FingerCluster, FiniteFloat, Keyboard, PositiveFloat, Preview, Ranged,
    ThumbCluster, Vec2, Vec3,
};
use three_d::egui::{
    Align, Checkbox, CollapsingHeader, DragValue, Frame, Layout, Margin, RichText, ScrollArea, Ui,
};

pub use config::Config;

const DRAG_SPEED: f64 = 0.1;

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

impl Show for Config {
    fn show(&mut self, ui: &mut Ui) {
        const MARGIN: Margin = Margin {
            left: 0.0,
            right: 8.0,
            top: 4.0,
            bottom: 8.0,
        };

        ui.add_space(8.0);
        ui.label(RichText::new("Configuration").strong().size(16.0));
        ScrollArea::vertical().show(ui, |ui| {
            Frame::default().inner_margin(MARGIN).show(ui, |ui| {
                self.preview.show(ui);
                self.finger_cluster.show(ui);
                self.thumb_cluster.show(ui);
                self.keyboard.show(ui);
                self.colors.show(ui);
            });
        });
    }
}

impl Show for Preview {
    fn show(&mut self, ui: &mut Ui) {
        parameters_section(ui, "Preview settings", |ui| {
            self.show_keys.show_with_name_and_description(
                ui,
                "Show keys",
                "Whether to show the keys during preview",
            );
            self.show_interface_pcb.show_with_name_and_description(
                ui,
                "Show interface PCB",
                "Whether to show the interface PCB during preview",
            );
            self.show_bottom_plate.show_with_name_and_description(
                ui,
                "Show bottom plate",
                "Whether to show the bottom plate during preview",
            );
            self.resolution.show_with_name_and_description(
                ui,
                "Resolution",
                "Resolution used for meshing, size of the smallest feature",
            );
        });
    }
}

impl Show for FingerCluster {
    fn show(&mut self, ui: &mut Ui) {
        parameters_section(ui, "Finger cluster", |ui| {
            self.rows.show_with_name_and_description(
                ui,
                "Rows",
                "Number of rows, automatic PCB generation is supported for 1 to 5 rows",
            );
            self.columns.show_with_name_and_description(
                ui,
                "Columns",
                "Settings per column, automatic PCB generation is supported for 2 to 6 columns",
            );
            self.key_distance.show_with_name_and_description(
                ui,
                "Key distance",
                "Distance between two neighbouring keys in X and Y direction",
            );
            self.home_row_index.show_with_name_and_description(
                ui,
                "Home row index",
                "Row index of the home row (usually 1)",
            );
        });
    }
}

impl Show for ThumbCluster {
    fn show(&mut self, ui: &mut Ui) {
        parameters_section(ui, "Thumb cluster", |ui| {
            self.keys.show_with_name_and_description(
                ui,
                "Keys",
                "Number of thumb keys, automatic PCB generation is supported for 1 to 6 keys",
            );
            self.curvature_angle.show_with_name_and_description(
                ui,
                "Curvature angle",
                "Thumb well curvature as angle between two neighbouring keys",
            );
            self.rotation.show_with_name_and_description(
                ui,
                "Rotation",
                "Rotation of the thumb cluster in relation to the finger cluster",
            );
            self.offset.show_with_name_and_description(
                ui,
                "Offset",
                "Offset of the thumb cluster in relation to the finger cluster",
            );
            self.key_distance.show_with_name_and_description(
                ui,
                "Key distance",
                "Distance between two neighbouring keys",
            );
            self.resting_key_index.show_with_name_and_description(
                ui,
                "Resting key index",
                "Index of the key the thumb is naturally resting on (usually the center key)",
            );
        });
    }
}

impl Show for Keyboard {
    fn show(&mut self, ui: &mut Ui) {
        parameters_section(ui, "Keyboard", |ui| {
            self.tilting_angle.show_with_name_and_description(
                ui,
                "Tilting angle",
                "Keyboard tilting angle along X and Y",
            );
            self.circumference_distance.show_with_name_and_description(
                ui,
                "Circumference distance",
                "Space around the key cluster",
            );
            self.rounding_radius.show_with_name_and_description(
                ui,
                "Rounding radius",
                "Rounding radius of the top keyboard edges",
            );
            self.shell_thickness.show_with_name_and_description(
                ui,
                "Shell thickness",
                "Thickness of the keyboard shell",
            );
            self.bottom_plate_thickness.show_with_name_and_description(
                ui,
                "Bottom plate thickness",
                "Thickness of the bottom plate",
            );
        });
    }
}

impl Show for Colors {
    fn show(&mut self, ui: &mut Ui) {
        parameters_section(ui, "Colors", |ui| {
            self.keyboard
                .show_with_name_and_description(ui, "Keyboard", "Color of the keyboard");
            self.keycap
                .show_with_name_and_description(ui, "Keycap", "Color of the keycaps");
            self.switch
                .show_with_name_and_description(ui, "Switch", "Color of the switches");
            self.matrix_pcb.show_with_name_and_description(
                ui,
                "Matrix PCB",
                "Color of the matrix PCB",
            );
            self.interface_pcb.show_with_name_and_description(
                ui,
                "Interface PCB",
                "Color of the interface PCB",
            );
            self.fpc_connector.show_with_name_and_description(
                ui,
                "FPC connector",
                "Color of the FPC connector",
            );
            self.background.show_with_name_and_description(
                ui,
                "Background",
                "Color of the background",
            );
        });
    }
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

impl<T: Show> Show for Vec2<T> {
    fn show(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.x.show(ui);
            self.y.show(ui);
        });
    }
}

impl<T: Show> Show for Vec3<T> {
    fn show(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.x.show(ui);
            self.y.show(ui);
            self.z.show(ui);
        });
    }
}

impl Show for FiniteFloat {
    fn show(&mut self, ui: &mut Ui) {
        let mut value = f64::from(*self);

        ui.add(
            DragValue::new(&mut value)
                .clamp_range(f64::MIN..=f64::MAX)
                .speed(DRAG_SPEED),
        );

        *self = value.try_into().expect("value should be finite");
    }
}

impl Show for PositiveFloat {
    fn show(&mut self, ui: &mut Ui) {
        let mut value = f64::from(*self);

        ui.add(
            DragValue::new(&mut value)
                .clamp_range(DRAG_SPEED..=f64::MAX)
                .speed(DRAG_SPEED),
        );

        *self = value
            .try_into()
            .expect("value should be finite and positive");
    }
}

impl<const LOWER: i8, const UPPER: i8> Show for Ranged<LOWER, UPPER> {
    fn show(&mut self, ui: &mut Ui) {
        let mut value = f64::from(*self);

        ui.add(
            DragValue::new(&mut value)
                .clamp_range(LOWER..=UPPER)
                .speed(DRAG_SPEED),
        );

        *self = value.try_into().expect("value should be within range");
    }
}

impl Show for Columns {
    fn show(&mut self, ui: &mut Ui) {}
}

fn parameters_section(ui: &mut Ui, title: &str, add_contents: impl FnOnce(&mut Ui)) {
    CollapsingHeader::new(RichText::new(title).size(14.0))
        .default_open(true)
        .show(ui, add_contents);
}
