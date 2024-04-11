use std::{
    env,
    ffi::OsStr,
    fs::{read_dir, read_to_string, File},
    io::Write,
    path::Path,
};

fn main() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR should always be set when building");
    let out_path = Path::new(&out_dir).join("assets.rs");
    let assets_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");

    let mut constants = String::new();
    let mut struct_definition = String::new();
    let mut struct_implementation = String::new();

    for entry in read_dir(assets_dir).expect("assets directory should exist") {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.extension() == Some(OsStr::new("obj")) {
                if let Some(name) = path.file_stem().map(|stem| stem.to_str()).flatten() {
                    let uppercase_name = name.to_uppercase();
                    let (positions, indices) = constants_from_path(&path);

                    struct_definition.push_str(&format!("\n    pub {name}: CpuMesh,"));
                    struct_implementation.push_str(&format!(
                        "
            {name}: CpuMesh {{
                positions: Positions::F32({uppercase_name}_POSITIONS),
                indices: Indices::U32({uppercase_name}_INDICES),
                ..Default::default()
            }},"
                    ));
                    constants.push_str(&format!(
                        "
const {uppercase_name}_POSITIONS: Vec<Vec3> = vec![
{positions}];
const {uppercase_name}_INDICES: Vec<u32> = vec![
{indices}];
"
                    ));
                }
            }
        }
    }

    let mut out_file = File::create(out_path).expect("output file should be createable in OUT_DIR");
    out_file
        .write_all(
            format!(
                "use three_d::{{CpuMesh, Indices, Positions, Vec3}};
{constants}
/// Fixed assets loaded from OBJ files.
pub struct Assets {{{struct_definition}
}}

impl Assets {{
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

fn constants_from_path(path: &Path) -> (String, String) {
    let mut positions = String::new();
    let mut indices = String::new();

    for line in read_to_string(path)
        .expect(&format!("failed to read file"))
        .lines()
    {
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
