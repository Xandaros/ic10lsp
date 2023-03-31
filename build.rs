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

    let infile = Path::new("stationpedia.txt");
    let contents = fs::read_to_string(infile).unwrap();

    for line in contents.lines() {
        let mut it = line.splitn(2, ' ');
        map_builder.entry(it.next().unwrap(), &format!("\"{}\"", it.next().unwrap()));
    }

    let output_file = File::create(dest_path).unwrap();
    let mut writer = BufWriter::new(&output_file);

    write!(
        &mut writer,
        "pub(crate) const HASH_NAME_LOOKUP: phf::Map<&'static str, &'static str> = {};\n",
        map_builder.build()
    )
    .unwrap();

    println!("cargo:rerun-if-changed=stationpedia.txt");
}
