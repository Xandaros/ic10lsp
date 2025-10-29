use std::{
    env,
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};

use itertools::Itertools;

fn write_stationpedia() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("stationpedia.rs");

    let mut name_map_builder = ::phf_codegen::Map::new();
    let mut desc_map_builder = ::phf_codegen::Map::new();
    let mut name_set_builder = ::phf_codegen::Set::new();
    let mut check_set = std::collections::HashSet::new();

    let infile = Path::new("data/stationpedia.txt");
    let contents = fs::read_to_string(infile).unwrap();

    for line in contents.lines().filter(|l| !l.trim().is_empty()) {
        let mut it = line.splitn(3, ' ');
        let hash = it.next().unwrap();
        let name = it.next().unwrap();
        let desc = it.next().unwrap_or("");
        name_map_builder.entry(hash, format!("\"{}\"", name));
        desc_map_builder.entry(hash, format!("\"{}\"", desc));

        if !check_set.contains(name) {
            name_set_builder.entry(name);
            check_set.insert(name);
        }
    }

    let output_file = File::create(dest_path).unwrap();
    let mut writer = BufWriter::new(&output_file);

    writeln!(
        &mut writer,
        "pub(crate) const HASH_NAME_LOOKUP: phf::Map<&'static str, &'static str> = {};",
        name_map_builder.build()
    )
    .unwrap();

    writeln!(
        &mut writer,
        "pub(crate) const HASH_DESC_LOOKUP: phf::Map<&'static str, &'static str> = {};",
        desc_map_builder.build()
    )
    .unwrap();

    writeln!(
        &mut writer,
        "pub(crate) const HASH_NAMES: phf::Set<&'static str> = {};",
        name_set_builder.build()
    )
    .unwrap();

    println!("cargo:rerun-if-changed=data/stationpedia.txt");
}

fn map_param_union(union: &str) -> String {
    match union {
        "d?" => "DEVICE".into(),
        "r?" => "REGISTER".into(),
        "r?|num" => "VALUE".into(),
        "r?|id" => "REGISTER_ID".into(),
        "r?|d?" => "REGISTER_DEVICE".into(),
        "d?|r?|id" => "DEVICE_ID".into(),
        _ => {
            println!("cargo::warning=Unknown param union {union:?}");
            "VALUE".into()
        }
    }
}

fn map_param_tag(tag: &str) -> String {
    match tag {
        "str" => "NAME".into(),
        "num" => "NUMBER".into(),
        "int" | "slotIndex" => format!(r#"VALUE.with_tag("{tag}")"#),
        "logicType" => "LOGIC_TYPE".into(),
        "logicSlotType" => "SLOT_LOGIC_TYPE".into(),
        "batchMode" => "BATCH_MODE".into(),
        "reagentMode" => "REAGENT_MODE".into(),
        "deviceHash" | "nameHash" => format!(r#"VALUE.with_tag("{tag}")"#),
        _ => {
            println!("cargo::warning=Unknown param tag {tag:?}");
            tag.into()
        }
    }
}

#[allow(unused_variables)]
fn format_instruction_params(params: &[&str]) -> String {
    let mut out_parts: Vec<String> = Vec::new();
    for param in params {
        if param.contains('(') {
            let mut param_iter = param.chars();
            let tag: String = param_iter
                .take_while_ref(|c| c.is_alphanumeric() || c == &'?' || c == &'|')
                .collect();
            let rest: Vec<_> = param_iter.collect();
            let param_union = match &&rest[..] {
                &['(', inner @ .., ')'] => {
                    let inner: String = inner.iter().collect();
                    Some(inner)
                }
                _ => {
                    let rest: String = rest.iter().collect();
                    if !rest.is_empty() {
                        println!(
                            "cargo::warning=WARNING: tag: {tag:?}, unknown non tag part {rest:?}"
                        );
                    }
                    None
                }
            };
            if let Some(union) = param_union {
                let union = map_param_union(&union);
                if !tag.is_empty() {
                    out_parts.push(format!(r#"{union}.with_tag("{tag}")"#));
                } else {
                    out_parts.push(union);
                }
            } else {
                let union = map_param_tag(&tag);
                if !tag.is_empty() {
                    out_parts.push(format!(r#"{union}.with_tag("{tag}")"#));
                } else {
                    out_parts.push(union);
                }
            }
        } else {
            let union = if param.contains(['?', '|']) {
                map_param_union(param)
            } else {
                map_param_tag(param)
            };
            out_parts.push(union);
        }
    }
    format!("InstructionSignature(&[{}])", out_parts.join(", "))
}

fn write_instructions() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("instructions.rs");
    let output_file = File::create(dest_path).unwrap();
    let mut writer = BufWriter::new(&output_file);

    let mut instruction_map_builder = ::phf_codegen::Map::new();
    let mut branch_instructions_builder = ::phf_codegen::Set::new();

    let in_sigfile = Path::new("data/instructions_sig.txt");
    let contents_sig = fs::read_to_string(in_sigfile).unwrap();

    for line in contents_sig.lines() {
        let mut it = line.split(' ');
        let instruction = it.next().unwrap();
        let ops: Vec<&str> = it.map(|s| s.trim()).collect();
        let sig = format_instruction_params(&ops);
        if instruction.starts_with(['j', 'b']) {
            branch_instructions_builder.entry(instruction);
        }
        instruction_map_builder.entry(instruction, sig);
    }

    let mut help_map_builder = ::phf_codegen::Map::new();
    let h_infile = Path::new("data/instructions_help.txt");
    let h_contents = fs::read_to_string(h_infile).unwrap();

    for line in h_contents.lines().filter(|l| !l.trim().is_empty()) {
        let mut it = line.splitn(2, ' ');
        let instruction = it.next().unwrap();
        let help = it.next().unwrap_or("").replace("\\n", "\n");
        help_map_builder.entry(instruction, format!("\"{}\"", help));
    }

    writeln!(
        &mut writer,
        "pub(crate) const INSTRUCTIONS: phf::Map<&'static str, InstructionSignature> = {};",
        instruction_map_builder.build()
    )
    .unwrap();

    writeln!(
        &mut writer,
        "pub(crate) const BRANCH_INSTRUCTIONS: phf::Set<&'static str> = {};",
        branch_instructions_builder.build()
    )
    .unwrap();

    println!("cargo:rerun-if-changed=data/instructions_sig.txt");

    writeln!(
        &mut writer,
        "pub(crate) const INSTRUCTION_DOCS: phf::Map<&'static str, &'static str> = {};",
        help_map_builder.build()
    )
    .unwrap();

    println!("cargo:rerun-if-changed=data/instructions_help.txt");
}

fn write_logictypes() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    let dest_path = Path::new(&out_dir).join("logictypes.rs");
    let output_file = File::create(dest_path).unwrap();
    let mut writer = BufWriter::new(&output_file);

    let mut logictype_set = ::phf_codegen::Set::new();
    let mut logictype_lookup_map_builder = ::phf_codegen::Map::new();
    let mut logictype_help_map_builder = ::phf_codegen::Map::new();
    let l_infile = Path::new("data/logictypes.txt");
    let l_contents = fs::read_to_string(l_infile).unwrap();

    for line in l_contents.lines().filter(|l| !l.trim().is_empty()) {
        let mut it = line.splitn(3, ' ');
        let name = it.next().unwrap();
        let val_str = it.next().unwrap();
        let val: Option<u16> = val_str.parse().ok();
        let help = it.next().unwrap_or("").replace("\\n", "\n");

        logictype_set.entry(name);
        if let Some(v) = val {
            logictype_lookup_map_builder.entry(v, format!("\"{}\"", name));
        }
        logictype_help_map_builder.entry(name, format!("\"{}\"", help));
    }

    let mut slotlogictype_set = ::phf_codegen::Set::new();
    let mut slotlogictype_lookup_map_builder = ::phf_codegen::Map::new();
    let mut slotlogictype_help_map_builder = ::phf_codegen::Map::new();
    let sl_infile = Path::new("data/slotlogictypes.txt");
    let sl_contents = fs::read_to_string(sl_infile).unwrap();

    for line in sl_contents.lines().filter(|l| !l.trim().is_empty()) {
        let mut it = line.splitn(3, ' ');
        let name = it.next().unwrap();
        let val_str = it.next().unwrap();
        let val: Option<u16> = val_str.parse().ok();
        let help = it.next().unwrap_or("").replace("\\n", "\n");

        slotlogictype_set.entry(name);
        if let Some(v) = val {
            slotlogictype_lookup_map_builder.entry(v, format!("\"{}\"", name));
        }
        slotlogictype_help_map_builder.entry(name, format!("\"{}\"", help));
    }

    writeln!(
        &mut writer,
        "pub(crate) const LOGIC_TYPES: phf::Set<&'static str> = {};",
        logictype_set.build()
    )
    .unwrap();

    writeln!(
        &mut writer,
        "pub(crate) const LOGIC_TYPE_LOOKUP: phf::Map<u16, &'static str> = {};",
        logictype_lookup_map_builder.build()
    )
    .unwrap();

    writeln!(
        &mut writer,
        "pub(crate) const LOGIC_TYPE_DOCS: phf::Map<&'static str, &'static str> = {};",
        logictype_help_map_builder.build()
    )
    .unwrap();

    println!("cargo:rerun-if-changed=data/logictypes.txt");

    writeln!(
        &mut writer,
        "pub(crate) const SLOT_LOGIC_TYPES: phf::Set<&'static str> = {};",
        slotlogictype_set.build()
    )
    .unwrap();

    writeln!(
        &mut writer,
        "pub(crate) const SLOT_TYPE_LOOKUP: phf::Map<u16, &'static str> = {};",
        slotlogictype_lookup_map_builder.build()
    )
    .unwrap();

    writeln!(
        &mut writer,
        "pub(crate) const SLOT_TYPE_DOCS: phf::Map<&'static str, &'static str> = {};",
        slotlogictype_help_map_builder.build()
    )
    .unwrap();

    println!("cargo:rerun-if-changed=data/slotlogictypes.txt");
}

fn write_modes() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    let dest_path = Path::new(&out_dir).join("modes.rs");
    let output_file = File::create(dest_path).unwrap();
    let mut writer = BufWriter::new(&output_file);

    let mut batchmode_set = ::phf_codegen::Set::new();
    let mut batchmode_lookup_map_builder = ::phf_codegen::Map::new();
    let mut batchmode_help_map_builder = ::phf_codegen::Map::new();
    let b_infile = Path::new("data/batchmodes.txt");
    let b_contents = fs::read_to_string(b_infile).unwrap();

    for line in b_contents.lines().filter(|l| !l.trim().is_empty()) {
        let mut it = line.splitn(3, ' ');
        let name = it.next().unwrap();
        let val_str = it.next().unwrap();
        let val: Option<u16> = val_str.parse().ok();
        let help = it.next().unwrap_or("").replace("\\n", "\n");

        batchmode_set.entry(name);
        if let Some(v) = val {
            batchmode_lookup_map_builder.entry(v, format!("\"{}\"", name));
        }
        batchmode_help_map_builder.entry(name, format!("\"{}\"", help));
    }

    let mut reagentmode_set = ::phf_codegen::Set::new();
    let mut reagentmode_lookup_map_builder = ::phf_codegen::Map::new();
    let mut reagentmode_help_map_builder = ::phf_codegen::Map::new();
    let r_infile = Path::new("data/reagentmodes.txt");
    let r_contents = fs::read_to_string(r_infile).unwrap();

    for line in r_contents.lines().filter(|l| !l.trim().is_empty()) {
        let mut it = line.splitn(3, ' ');
        let name = it.next().unwrap();
        let val_str = it.next().unwrap();
        let val: Option<u16> = val_str.parse().ok();
        let help = it.next().unwrap_or("").replace("\\n", "\n");

        reagentmode_set.entry(name);
        if let Some(v) = val {
            reagentmode_lookup_map_builder.entry(v, format!("\"{}\"", name));
        }
        reagentmode_help_map_builder.entry(name, format!("\"{}\"", help));
    }

    writeln!(
        &mut writer,
        "pub(crate) const BATCH_MODES: phf::Set<&'static str> = {};",
        batchmode_set.build()
    )
    .unwrap();

    writeln!(
        &mut writer,
        "pub(crate) const BATCH_MODE_LOOKUP: phf::Map<u16, &'static str> = {};",
        batchmode_lookup_map_builder.build()
    )
    .unwrap();

    writeln!(
        &mut writer,
        "pub(crate) const BATCH_MODE_DOCS: phf::Map<&'static str, &'static str> = {};",
        batchmode_help_map_builder.build()
    )
    .unwrap();

    println!("cargo:rerun-if-changed=data/batchmodes.txt");

    writeln!(
        &mut writer,
        "pub(crate) const REAGENT_MODES: phf::Set<&'static str> = {};",
        reagentmode_set.build()
    )
    .unwrap();

    writeln!(
        &mut writer,
        "pub(crate) const REAGENT_MODE_LOOKUP: phf::Map<u16, &'static str> = {};",
        reagentmode_lookup_map_builder.build()
    )
    .unwrap();

    writeln!(
        &mut writer,
        "pub(crate) const REAGENT_MODE_DOCS: phf::Map<&'static str, &'static str> = {};",
        reagentmode_help_map_builder.build()
    )
    .unwrap();

    println!("cargo:rerun-if-changed=data/reagentmodes.txt");
}

fn write_constants() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    let dest_path = Path::new(&out_dir).join("constants.rs");
    let output_file = File::create(dest_path).unwrap();
    let mut writer = BufWriter::new(&output_file);

    let mut constants_set = ::phf_codegen::Set::new();
    let mut constants_help_map_builder = ::phf_codegen::Map::new();
    let infile = Path::new("data/constants.txt");
    let contents = fs::read_to_string(infile).unwrap();

    for line in contents.lines().filter(|l| !l.trim().is_empty()) {
        let mut it = line.splitn(2, ' ');
        let name = it.next().unwrap();
        let help = it.next().unwrap_or("").replace("\\n", "\n");

        constants_set.entry(name);
        constants_help_map_builder.entry(name, format!("\"{}\"", help));
    }

    writeln!(
        &mut writer,
        "pub(crate) const CONSTANTS: phf::Set<&'static str> = {};",
        constants_set.build()
    )
    .unwrap();

    writeln!(
        &mut writer,
        "pub(crate) const CONSTANTS_DOCS: phf::Map<&'static str, &'static str> = {};",
        constants_help_map_builder.build()
    )
    .unwrap();

    println!("cargo:rerun-if-changed=data/constants.txt");
}

fn write_enums() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    let dest_path = Path::new(&out_dir).join("enums.rs");
    let output_file = File::create(dest_path).unwrap();
    let mut writer = BufWriter::new(&output_file);

    let mut enums_set = ::phf_codegen::Set::new();
    let mut enums_help_map_builder = ::phf_codegen::Map::new();
    let mut enums_lookup_map_builder = ::phf_codegen::Map::new();
    let mut check_set = std::collections::HashSet::new();
    let e_infile = Path::new("data/enums.txt");
    let e_contents = fs::read_to_string(e_infile).unwrap();

    for line in e_contents.lines().filter(|l| !l.trim().is_empty()) {
        let mut it = line.splitn(3, ' ');
        let name = it.next().unwrap();
        let val_str = it.next().unwrap();
        let val: Option<u32> = val_str.parse().ok();
        let help = it.next().unwrap_or("").replace("\\n", "\n");

        if !check_set.contains(name) {
            enums_set.entry(name);
            check_set.insert(name);
        }

        if let Some(v) = val {
            enums_lookup_map_builder.entry(name, format!("{}u32", v));
        }
        enums_help_map_builder.entry(name, format!("\"{}\"", help));
    }

    writeln!(
        &mut writer,
        "pub(crate) const ENUMS: phf::Set<&'static str> = {};",
        enums_set.build()
    )
    .unwrap();

    writeln!(
        &mut writer,
        "pub(crate) const ENUM_LOOKUP: phf::Map<&'static str, u32> = {};",
        enums_lookup_map_builder.build()
    )
    .unwrap();

    writeln!(
        &mut writer,
        "pub(crate) const ENUM_DOCS: phf::Map<&'static str, &'static str> = {};",
        enums_help_map_builder.build()
    )
    .unwrap();

    println!("cargo:rerun-if-changed=data/enums.txt");
    println!("cargo:rerun-if-changed=data/enum_help.txt");
}

fn main() {
    write_stationpedia();
    write_instructions();
    write_logictypes();
    write_modes();
    write_constants();
    write_enums();
}
