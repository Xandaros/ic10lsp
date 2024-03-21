import xml.etree.ElementTree as ET
import argparse
from pathlib import Path
import sys
import re
from itertools import chain
import struct
import binascii

def main():
    arg_parser = argparse.ArgumentParser(
            prog="ic10lsp Data Generator",
            description="Generate instructions, enums, and docs for lsp",
            epilog="Point at the Stationeers install and go!",
            formatter_class=argparse.ArgumentDefaultsHelpFormatter)
    arg_parser.add_argument("path", help="Path to Stationeers installation")
    arg_parser.add_argument("--lang", help="language to extract from (ie. english)", default="english")
    args = arg_parser.parse_args()
    install_path = Path(args.path)
    if install_path.match("Stationeers/*.exe") or install_path.match("Stationeers/rocketstation_Data"):
        install_path = install_path.parent
    elif install_path.name == "Stationeers":
        pass
    elif (install_path / "Stationeers").is_dir():
        install_path = install_path / "Stationeers"
    
    data_path = install_path / "rocketstation_Data" / "StreamingAssets" / "Language" 
    if not data_path.is_dir():

        print(f"Invalid install path. {install_path} does not point to a valid Stationeers installation")
        arg_parser.print_help()
        sys.exit(1)

    lang = args.lang
    if not (data_path / f"{lang}.xml").is_file():
        print("Language file '{lang}.xml' does not exist. can not pull help strings.")
        sys.exit(2)

    extract_data(data_path, lang)

def extract_data(data_path: Path, language: str):
    tree = ET.parse(data_path / f"{language}.xml")
    root = tree.getroot()
    interface = root.find("Interface")
    strings = root.find("GameStrings")
    colors = root.find("Colors")
    elms = [elm for elm in (interface, strings, colors) if elm is not None ]

    
    logic_type = re.compile(r"LogicType(\w+)")
    logic_slot_type = re.compile(r"LogicSlotType(\w+)")
    script_command = re.compile(r"ScriptCommand(\w+)")
    script_desc = re.compile(r"ScriptDescription(\w+)")
    color = re.compile(r"Color(\w+)")
    operation_help_strings: dict[str, str] = {}
    enum_help_strings: dict[str, str] = {}
    logic_types: dict[str, tuple[int|None, str]] = {}
    slot_logic_types: dict[str, tuple[int|None, str]] = {}
    for record in chain.from_iterable(elms):
        key = record.find("Key")
        value = record.find("Value")
        if key is None or value is None:
            continue
        key = key.text
        value = value.text
        if key is None or value is None:
            continue
        if match := logic_type.match(key):
            enum_help_strings[f"LogicType.{match.group(1)}"] = value
            logic_types[match.group(1)] = (None, value)
        if match := logic_slot_type.match(key):
            enum_help_strings[f"LogicSlotType.{match.group(1)}"] = value
            slot_logic_types[match.group(1)] = (None, value)
        if match := color.match(key):
            enum_help_strings[f"Color.{match.group(1)}"] = value
        if match := script_command.match(key):
            operation_help_strings[f"{match.group(1).lower()}"] = value
        if match := script_desc.match(key):
            operation_help_strings[f"{match.group(1).lower()}"] = value
    
    op_help_path = Path("data") / "instructions_help.txt"
    with op_help_path.open(mode="w") as f:
        for key, val in sorted(operation_help_strings.items()):
            f.write("{} {}\n".format(key, val.replace("\r", "").replace("\n", "\\n")))

    enum_help_path = Path("data") / "enum_help.txt"
    with enum_help_path.open(mode="w") as f:
        for key, val in sorted(enum_help_strings.items()):
            f.write("{} {}\n".format(key, val.replace("\r", "").replace("\n", "\\n")))

    stationpedia: dict[str, tuple[str, str | None]] = {}
    things = root.find("Things")
    reagents = root.find("Reagents")
    hashables = [elm for elm in (things, reagents) if elm is not None]
    for record in chain.from_iterable(hashables):
        key = record.find("Key")
        value = record.find("Value")
        if key is None or value is None:
            continue
        key = key.text
        value = value.text
        if key is None:
            continue
        crc = binascii.crc32(key.encode('utf-8'))
        crc_s = struct.unpack("i", struct.pack("I", crc))[0]
        stationpedia[crc_s] = (key, value)

    hashables_path = Path("data") / "stationpedia.txt"
    with hashables_path.open(mode="w") as f:
        for key, val in sorted(stationpedia.items(), key=lambda i: i[1][0]):
            name = val[0]
            desc = val[1] if val[1] is not None else ""
            f.write("{} {} {}\n".format(key, name, desc.replace("\r", "").replace("\n", "\\n")))

    enums: dict[str, int] = {}
    enums_path = Path("data") / "enums.txt"
    with enums_path.open(mode="r") as f:
        lines = f.readlines()
        for l in filter(lambda l: l.strip(), lines):
            name, value = l.strip().split(" ", 1)
            enums[name] = int(value)
    for name, value in enums.items():
        if name.startswith("LogicType"):
            name = name.split(".", 1)[1]
            if name not in logic_types:
                logic_types[name] = (value, "")
            else:
                help = logic_types[name][1]
                logic_types[name] = (value, help)
        elif name.startswith("LogicSlotType"):
            name = name.split(".", 1)[1]
            if name not in slot_logic_types:
                slot_logic_types[name] = (value, "")
            else:
                help = slot_logic_types[name][1]
                slot_logic_types[name] = (value, help)

    logic_types_path = Path("data") / "logictypes.txt"
    with logic_types_path.open(mode="w") as f:
        for t, (v, help) in sorted(logic_types.items()):
            f.write(f"{t} {v} {help.replace("\r", "").replace("\n", "\\n")}\n")
    slot_logic_types_path = Path("data") / "slotlogictypes.txt"
    with slot_logic_types_path.open(mode="w") as f:
        for t, (v, help) in sorted(slot_logic_types.items()):
            f.write(f"{t} {v} {help.replace("\r", "").replace("\n", "\\n")}\n")

if __name__ == "__main__":
    main()
