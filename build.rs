use std::{env, fmt::Write, fs, path::Path};

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("stationpedia.rs");

    let mut output =
        "pub(crate) const HASH_NAME_LOOKUP: phf::Map<&'static str, &'static str> = phf_map! {\n"
            .to_string();

    let infile = Path::new("stationpedia.txt");
    let contents = fs::read_to_string(infile).unwrap();

    for line in contents.lines() {
        let (hash, name) = {
            let mut it = line.splitn(2, ' ');
            (it.next().unwrap(), it.next().unwrap())
        };

        writeln!(&mut output, "\"{hash}\" => \"{name}\",").unwrap();
    }
    writeln!(&mut output, "}};").unwrap();

    fs::write(dest_path, output).unwrap();
    println!("cargo:rerun-if-changed=stationpedia.txt");
}
