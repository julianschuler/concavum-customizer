//! The build script for the `viewer` crate converts the OBJ asset files at compile time to a rust struct.

use std::{
    env,
    ffi::OsStr,
    fs::{read_dir, read_to_string, File},
    io::Write,
    path::Path,
};

fn main() {
    println!("cargo::rerun-if-changed=assets");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR should always be set when building");
    let out_path = Path::new(&out_dir).join("assets.rs");
    let assets_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");

    let mut statics = String::new();
    let mut struct_definition = String::new();
    let mut struct_implementation = String::new();

    for entry in read_dir(assets_dir)
        .expect("assets directory should exist")
        .flatten()
    {
        let path = entry.path();
        if path.extension() == Some(OsStr::new("obj")) {
            if let Some(name) = path.file_stem().and_then(|stem| stem.to_str()) {
                let uppercase_name = name.to_uppercase();
                let (positions, indices) = statics_from_path(&path);

                struct_definition.push_str(&format!("\n    pub {name}: CpuMesh,"));
                struct_implementation.push_str(&format!(
                    "
            {name}: CpuMesh {{
                positions: Positions::F32({uppercase_name}_POSITIONS.to_vec()),
                indices: Indices::U32({uppercase_name}_INDICES.to_vec()),
                ..Default::default()
            }},"
                ));
                statics.push_str(&format!(
                    "
#[allow(clippy::approx_constant)]
#[allow(clippy::unreadable_literal)]
static {uppercase_name}_POSITIONS: [Vec3; {}] = [
{}];
static {uppercase_name}_INDICES: [u32; {}] = [
{}];
",
                    { positions.length },
                    { positions.entries },
                    { indices.length * 3 },
                    { indices.entries },
                ));
            }
        }
    }

    let mut out_file = File::create(out_path).expect("output file should be createable in OUT_DIR");
    out_file
        .write_all(
            format!(
                "use three_d::{{CpuMesh, Indices, Positions, Vec3}};
{statics}
/// Fixed assets loaded from OBJ files.
pub struct Assets {{{struct_definition}
}}

impl Assets {{
    /// Creates the assets from the compile-time statics.
    pub fn new() -> Self {{
        Self {{{struct_implementation}
        }}
    }}
}}
"
            )
            .as_bytes(),
        )
        .expect("writing to the output file should always succeed");
}

fn statics_from_path(path: &Path) -> (Static, Static) {
    let mut positions = Static::positions();
    let mut indices = Static::indicies();

    for line in read_to_string(path).expect("failed to read file").lines() {
        let mut iter = line.split_whitespace();
        match iter.next() {
            Some("v") => {
                let a = iter.next().expect("missing coordinate");
                let b = iter.next().expect("missing coordinate");
                let c = iter.next().expect("missing coordinate");
                positions.push_str(&format!("    Vec3::new({a}f32, {b}f32, {c}f32),\n"));
            }
            Some("f") => {
                let a = iter.next().expect("missing vertex");
                let b = iter.next().expect("missing vertex");
                let c = iter.next().expect("missing vertex");
                indices.push_str(&format!("    {a}, {b}, {c},\n"));
            }
            _ => {}
        }
    }

    (positions, indices)
}

struct Static {
    entries: String,
    length: usize,
}

impl Static {
    fn positions() -> Self {
        // OBJ files are 1-indexed, add an unused entry here to avoid parsing values
        let entries = "    Vec3::new(0f32, 0f32, 0f32),\n".to_owned();

        Self { entries, length: 1 }
    }

    fn indicies() -> Self {
        Self {
            entries: String::new(),
            length: 0,
        }
    }

    fn push_str(&mut self, string: &str) {
        self.entries.push_str(string);
        self.length += 1;
    }
}
