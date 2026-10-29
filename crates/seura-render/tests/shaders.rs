//! The preview's copies of the shaders (technical design, section 6): the
//! WGSL source for WebGPU, and naga's GLSL ES translation of it for WebGL2.
//! Both are generated from `shaders/` and committed, so the UI builds
//! without Rust; they're never edited by hand.
//!
//! Run with `SEURA_UPDATE_SHADERS=1` (or `npm run shaders` in ui/) to
//! regenerate them.

use std::fs;
use std::path::PathBuf;

use naga::back::glsl;
use naga::valid::{Capabilities, ValidationFlags, Validator};

const HEADER: &str = "Generated from crates/seura-render/shaders by `npm run shaders`. Don't edit.";

fn generated() -> Vec<(&'static str, String)> {
    let module = naga::front::wgsl::parse_str(seura_render::SHADER).expect("the WGSL parses");
    // WebGL2 has no compute or storage, so validate for the downlevel set.
    let info = Validator::new(ValidationFlags::all(), Capabilities::empty())
        .validate(&module)
        .expect("the WGSL is valid");

    let options = glsl::Options {
        version: glsl::Version::new_gles(300),
        // The WGSL's coordinates are kept as written: the preview draws
        // straight to the canvas, whose top is up in both APIs.
        writer_flags: glsl::WriterFlags::empty(),
        ..Default::default()
    };
    let mut files = vec![(
        "shader.wgsl",
        format!("// {HEADER}\n\n{}", seura_render::SHADER),
    )];
    let mut names = Vec::new();
    for (stage, entry_point, file) in [
        (naga::ShaderStage::Vertex, "vs_main", "shader.vert"),
        (naga::ShaderStage::Fragment, "fs_main", "shader.frag"),
    ] {
        let pipeline = glsl::PipelineOptions {
            shader_stage: stage,
            entry_point: entry_point.into(),
            multiview: None,
        };
        let mut source = String::new();
        let reflection = glsl::Writer::new(
            &mut source,
            &module,
            &info,
            &options,
            &pipeline,
            naga::proc::BoundsCheckPolicies::default(),
        )
        .and_then(|mut writer| writer.write())
        .unwrap_or_else(|error| panic!("{entry_point} translates to GLSL ES: {error}"));
        if stage == naga::ShaderStage::Fragment {
            // What the WebGL2 renderer binds by name.
            let block = reflection
                .uniforms
                .values()
                .next()
                .expect("one uniform block");
            let sampler = reflection
                .texture_mapping
                .keys()
                .next()
                .expect("one texture");
            names.push(format!(
                "{{\n  \"note\": \"{HEADER}\",\n  \"paramsBlock\": \"{block}\",\n  \"photo\": \"{sampler}\"\n}}\n"
            ));
        }
        // The version line must come first, so the note goes after it.
        let (version, rest) = source.split_once('\n').expect("a version line");
        files.push((file, format!("{version}\n// {HEADER}\n{rest}")));
    }
    files.push(("gl-names.json", names.remove(0)));
    files.push(("params.json", params_layout(&module)));
    files
}

/// Where each of `Params`' fields is in the uniform block, as naga lays it
/// out, for the preview to write them (`ui/src/preview/backend.ts`).
fn params_layout(module: &naga::Module) -> String {
    let (members, span) = module
        .types
        .iter()
        .find_map(|(_, ty)| match &ty.inner {
            naga::TypeInner::Struct { members, span } if ty.name.as_deref() == Some("Params") => {
                Some((members, *span))
            }
            _ => None,
        })
        .expect("a Params struct");
    let fields: Vec<String> = members
        .iter()
        .map(|member| {
            let kind = match module.types[member.ty].inner {
                naga::TypeInner::Scalar(naga::Scalar {
                    kind: naga::ScalarKind::Float,
                    width: 4,
                }) => "f32",
                naga::TypeInner::Scalar(naga::Scalar {
                    kind: naga::ScalarKind::Uint,
                    width: 4,
                }) => "u32",
                ref other => panic!("Params holds only f32 and u32, not {other:?}"),
            };
            format!(
                "    {{ \"name\": \"{}\", \"offset\": {}, \"type\": \"{kind}\" }}",
                member.name.as_deref().expect("a named field"),
                member.offset
            )
        })
        .collect();
    format!(
        "{{
  \"note\": \"{HEADER}\",
  \"size\": {span},
  \"fields\": [
{}
  ]
}}
",
        fields.join(
            ",
"
        )
    )
}

#[test]
fn preview_shaders_are_current() {
    let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../ui/src/preview/generated");
    let update = std::env::var_os("SEURA_UPDATE_SHADERS").is_some();
    if update {
        fs::create_dir_all(&folder).unwrap();
    }
    for (name, contents) in generated() {
        let path = folder.join(name);
        if update {
            fs::write(&path, &contents).unwrap();
            continue;
        }
        let current = fs::read_to_string(&path)
            .unwrap_or_default()
            .replace("\r\n", "\n");
        assert!(
            current == contents,
            "ui/src/preview/generated/{name} is out of date. Run `npm run shaders` in ui/ to regenerate it."
        );
    }
}
