use serde::Serialize;

use crate::{
    footprints::Footprint,
    primitives::{Point, Uuid},
    serializer::Serializer,
    unit::{IntoUnit, Length},
};

/// A KiCAD PCB.
#[derive(Serialize)]
pub struct KicadPcb {
    version: u32,
    generator: String,
    general: General,
    paper: String,
    layers: Layers,
    setup: SetupSettings,
    nets: Vec<Net>,
    footprints: Vec<Footprint>,
    segments: Vec<Segment>,
    arcs: Vec<Arc>,
}

impl Default for KicadPcb {
    fn default() -> Self {
        Self::new(1.6.mm())
    }
}

impl KicadPcb {
    /// Creates an empty KiCAD PCB with the given thickness.
    #[must_use]
    pub fn new(thickness: Length) -> Self {
        Self {
            #[allow(clippy::unreadable_literal)]
            version: 20240108,
            generator: "concavum_customizer".to_owned(),
            general: General {
                thickness,
                legacy_teardrops: false,
            },
            paper: "A4".to_owned(),
            layers: Layers::default(),
            setup: SetupSettings::default(),
            nets: vec![Net(0, String::new())],
            footprints: Vec::default(),
            segments: Vec::default(),
            arcs: Vec::default(),
        }
    }

    /// Serializes the PCB to the KiCAD board file format.
    ///
    /// # Panics
    ///
    /// Panics if the PCB could not be serialized.
    #[must_use]
    pub fn to_board_file(&self) -> String {
        let mut serializer = Serializer::new("kicad_pcb");

        self.serialize(&mut serializer)
            .expect("PCB should always be serializable");

        serializer.finish()
    }

    /// Adds a segment to the PCB.
    pub fn add_segment(
        &mut self,
        start: Point,
        end: Point,
        width: Length,
        layer: &'static str,
        net: u32,
    ) {
        self.segments.push(Segment {
            start,
            end,
            width,
            layer,
            net,
            uuid: Uuid::new(),
        });
    }

    /// Adds an arc to the PCB.
    pub fn add_arc(
        &mut self,
        start: Point,
        mid: Point,
        end: Point,
        width: Length,
        layer: &'static str,
        net: u32,
    ) {
        self.arcs.push(Arc {
            start,
            mid,
            end,
            width,
            layer,
            net,
            uuid: Uuid::new(),
        });
    }

    /// Adds the given footprint to the PCB.
    pub fn add_footprint(&mut self, footprint: Footprint) {
        self.footprints.push(footprint);
    }
}

#[derive(Serialize)]
struct General {
    thickness: Length,
    legacy_teardrops: bool,
}

#[derive(Serialize)]
struct Layers(Vec<Layer>);

#[derive(Serialize)]
struct Layer(u32, &'static str, LayerType, Option<&'static str>);

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum LayerType {
    Signal,
    User,
}

impl Default for Layers {
    fn default() -> Self {
        Self(vec![
            Layer(0, "F.Cu", LayerType::Signal, None),
            Layer(31, "B.Cu", LayerType::Signal, None),
            Layer(32, "B.Adhes", LayerType::User, Some("B.Adhesive")),
            Layer(33, "F.Adhes", LayerType::User, Some("F.Adhesive")),
            Layer(34, "B.Paste", LayerType::User, None),
            Layer(35, "F.Paste", LayerType::User, None),
            Layer(36, "B.SilkS", LayerType::User, Some("B.Silkscreen")),
            Layer(37, "F.SilkS", LayerType::User, Some("F.Silkscreen")),
            Layer(38, "B.Mask", LayerType::User, None),
            Layer(39, "F.Mask", LayerType::User, None),
            Layer(40, "Dwgs.User", LayerType::User, Some("User.Drawings")),
            Layer(41, "Cmts.User", LayerType::User, Some("User.Comments")),
            Layer(42, "Eco1.User", LayerType::User, Some("User.Eco1")),
            Layer(43, "Eco2.User", LayerType::User, Some("User.Eco2")),
            Layer(44, "Edge.Cuts", LayerType::User, None),
            Layer(45, "Margin", LayerType::User, None),
            Layer(46, "B.CrtYd", LayerType::User, Some("B.Courtyard")),
            Layer(47, "F.CrtYd", LayerType::User, Some("F.Courtyard")),
            Layer(48, "B.Fab", LayerType::User, None),
            Layer(49, "F.Fab", LayerType::User, None),
            Layer(50, "User.1", LayerType::User, None),
            Layer(51, "User.2", LayerType::User, None),
            Layer(52, "User.3", LayerType::User, None),
            Layer(53, "User.4", LayerType::User, None),
            Layer(54, "User.5", LayerType::User, None),
            Layer(55, "User.6", LayerType::User, None),
            Layer(56, "User.7", LayerType::User, None),
            Layer(57, "User.8", LayerType::User, None),
            Layer(58, "User.9", LayerType::User, None),
        ])
    }
}

#[derive(Serialize, Default)]
struct SetupSettings {
    pad_to_mask_clearance: Length,
    allow_soldermask_bridges_in_footprints: bool,
    pcbplotparams: PlotParameters,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Serialize)]
struct PlotParameters {
    layerselection: u64,
    plot_on_all_layers_selection: u64,
    disableapertmacros: bool,
    usegerberextensions: bool,
    usegerberattributes: bool,
    usegerberadvancedattributes: bool,
    creategerberjobfile: bool,
    dashed_line_dash_ratio: f32,
    dashed_line_gap_ratio: f32,
    svgprecision: u32,
    plotframeref: bool,
    viasonmask: bool,
    mode: u8,
    useauxorigin: bool,
    hpglpennumber: u32,
    hpglpenspeed: u32,
    hpglpendiameter: f32,
    pdf_front_fp_property_popups: bool,
    pdf_back_fp_property_popups: bool,
    dxfpolygonmode: bool,
    dxfimperialunits: bool,
    dxfusepcbnewfont: bool,
    psnegative: bool,
    psa4output: bool,
    plotreference: bool,
    plotvalue: bool,
    plotfptext: bool,
    plotinvisibletext: bool,
    sketchpadsonfab: bool,
    subtractmaskfromsilk: bool,
    outputformat: u32,
    mirror: bool,
    drillshape: u32,
    scaleselection: u32,
    outputdirectory: String,
}

impl Default for PlotParameters {
    fn default() -> Self {
        Self {
            layerselection: 0x000_0030_ffff_ffff,
            plot_on_all_layers_selection: 0x000_0000_0000_0000,
            disableapertmacros: false,
            usegerberextensions: false,
            usegerberattributes: true,
            usegerberadvancedattributes: true,
            creategerberjobfile: true,
            dashed_line_dash_ratio: 12.0,
            dashed_line_gap_ratio: 3.0,
            svgprecision: 4,
            plotframeref: false,
            viasonmask: false,
            mode: 1,
            useauxorigin: false,
            hpglpennumber: 1,
            hpglpenspeed: 20,
            hpglpendiameter: 15.0,
            pdf_front_fp_property_popups: true,
            pdf_back_fp_property_popups: true,
            dxfpolygonmode: true,
            dxfimperialunits: true,
            dxfusepcbnewfont: true,
            psnegative: false,
            psa4output: false,
            plotreference: true,
            plotvalue: true,
            plotfptext: true,
            plotinvisibletext: false,
            sketchpadsonfab: false,
            subtractmaskfromsilk: false,
            outputformat: 1,
            mirror: false,
            drillshape: 0,
            scaleselection: 1,
            outputdirectory: String::new(),
        }
    }
}

#[derive(Serialize)]
struct Net(u32, String);

#[derive(Serialize)]
struct Segment {
    start: Point,
    end: Point,
    width: Length,
    layer: &'static str,
    net: u32,
    uuid: Uuid,
}

#[derive(Serialize)]
struct Arc {
    start: Point,
    mid: Point,
    end: Point,
    width: Length,
    layer: &'static str,
    net: u32,
    uuid: Uuid,
}
