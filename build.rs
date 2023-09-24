use std::{
    env,
    fs::{self, File},
    io::BufWriter,
    io::Write,
    path::Path,
};

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("stationpedia.rs");

    let mut map_builder = ::phf_codegen::Map::new();
    let mut set_builder = ::phf_codegen::Set::new();
    let mut check_set = std::collections::HashSet::new();

    let infile = Path::new("stationpedia.txt");
    let contents = fs::read_to_string(infile).unwrap();

    for line in contents.lines() {
        let mut it = line.splitn(2, ' ');
        let hash = it.next().unwrap();
        let name = it.next().unwrap();
        map_builder.entry(hash, &format!("\"{}\"", name));

        if !check_set.contains(name) {
            set_builder.entry(name);
            check_set.insert(name);
        }
    }

    let output_file = File::create(dest_path).unwrap();
    let mut writer = BufWriter::new(&output_file);

    write!(
        &mut writer,
        "pub(crate) const HASH_NAME_LOOKUP: phf::Map<&'static str, &'static str> = {};\n",
        map_builder.build()
    )
    .unwrap();

    write!(
        &mut writer,
        "pub(crate) const HASH_NAMES: phf::Set<&'static str> = {};\n",
        set_builder.build()
    )
    .unwrap();

    println!("cargo:rerun-if-changed=stationpedia.txt");
}
