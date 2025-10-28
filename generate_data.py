#!/usr/bin/env python3

import argparse
import binascii
import json
import re
import struct
import sys
import xml.etree.ElementTree as ET
from collections import defaultdict
from itertools import chain, filterfalse, tee
from pathlib import Path
from pprint import pprint
from typing import Callable, Iterable, TypedDict, TypeVar

translation_regex = re.compile(r"<N:([A-Z]{2}):(\w+)>")

def replace_translation(m: re.Match[str]) -> str:
    match m.groups():
        case (_code, key):
            return key
        case _:
            return m.string


def trans(s: str) -> str:
    return re.sub(translation_regex, replace_translation, s)


def intOrNone(val):
    try:
        return int(val)
    except ValueError:
        return None


class ScriptCommand(TypedDict):
    desc: str
    example: str


class ScriptConstant(TypedDict):
    desc: str
    value: str


class StationpediaPage(TypedDict):
    Key: str
    Title: str
    Description: str
    PrefabName: str
    PrefabHash: int


class ReagentEntry(TypedDict):
    Id: int
    Hash: int
    Unit: str
    IsOrganic: bool


class Stationpedia(TypedDict):
    version: str
    pages: list[StationpediaPage]
    core_prefabs: list
    reagents: dict[str, ReagentEntry]
    scriptCommands: dict[str, ScriptCommand]
    scriptConstants: dict[str, ScriptConstant]


class EnumEntry(TypedDict):
    value: int
    deprecated: bool
    description: str


class EnumListing(TypedDict):
    enumName: str
    values: dict[str, EnumEntry]


class Enums(TypedDict):
    scriptEnums: dict[str, EnumListing]
    basicEnums: dict[str, EnumListing]


class HelpPatches(TypedDict):
    operations: dict[str, str]
    constants: dict[str, str]
    batchmodes: dict[str, str]
    reagentmodes: dict[str, str]
    enums: dict[str, str]


T = TypeVar("T")
D = TypeVar("D")


def partition(
    iter: Iterable[T], pred: Callable[[T], bool]
) -> tuple[Iterable[T], Iterable[T]]:
    t1, t2 = tee(iter)
    return filterfalse(pred, t1), filter(pred, t2)


def partition_map(
    iter: Iterable[T], pred: Callable[[T], bool], maping: Callable[[T], D]
) -> tuple[Iterable[D], Iterable[D]]:
    t1, t2 = partition(iter, pred)
    return map(maping, t1), map(maping, t2)


def main():
    arg_parser = argparse.ArgumentParser(
        description="Generate instructions, enums, and docs for lsp.\n\nWorks best when using https://github.com/Ryex/StationeersStationpediaExtractor",
        epilog="Point at the Stationeers install and go!",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    arg_parser.add_argument(
        "path",
        help="Path to Stationeers installation or location of Stationpedia.json and Enums.json",
    )
    args = arg_parser.parse_args()
    install_path = Path(args.path)
    if install_path.match("Stationeers/*.exe") or install_path.match(
        "Stationeers/rocketstation_Data"
    ):
        install_path = install_path.parent
    elif install_path.name == "Stationeers":
        pass
    elif (install_path / "Stationeers").is_dir():
        install_path = install_path / "Stationeers"

    if (install_path / "Stationpedia").is_dir():
        install_path = install_path / "Stationpedia"

    enums_path = install_path / "Enums.json"
    stationpedia_path = install_path / "Stationpedia.json"
    for path in [stationpedia_path, enums_path]:
        if not path.exists():
            print(f"Invalid data path. '{path}' does nto exist")
            arg_parser.print_help()
            sys.exit(1)

    extract_data(install_path)


def extract_data(install_path):
    enums_path = install_path / "Enums.json"
    stationpedia_path = install_path / "Stationpedia.json"

    operations: dict[str, ScriptCommand] = {}
    constants: dict[str, ScriptConstant] = {}
    enums: dict[str, tuple[int | None, str]] = {}
    logictypes: dict[str, tuple[int | None, str]] = {}
    slotlogictypes: dict[str, tuple[int | None, str]] = {}
    reagentmodes: dict[str, tuple[int | None, str]] = {}
    batchmodes: dict[str, tuple[int | None, str]] = {}

    exported_stationpedia: Stationpedia = (
        None  # pyright: ignore[reportRedeclaration, reportAssignmentType]
    )
    with stationpedia_path.open(mode="r") as f:
        exported_stationpedia = json.load(f)

    exported_enum_listing: Enums = {"basicEnums": {}, "scriptEnums": {}}

    with enums_path.open(mode="r") as f:
        exported_enum_listing = json.load(f)

    exported_logictypes = map(
        lambda enum: (enum[0], (intOrNone(enum[1]["value"]), trans(enum[1]["description"]))),
        exported_enum_listing["scriptEnums"]["LogicType"]["values"].items(),
    )
    exported_slotlogictypes = map(
        lambda enum: (enum[0], (intOrNone(enum[1]["value"]), trans(enum[1]["description"]))),
        exported_enum_listing["scriptEnums"]["LogicSlotType"]["values"].items(),
    )
    exported_reagentmodes = map(
        lambda enum: (enum[0], (intOrNone(enum[1]["value"]), trans(enum[1]["description"]))),
        exported_enum_listing["scriptEnums"]["LogicReagentMode"]["values"].items(),
    )
    exported_batchmodes = map(
        lambda enum: (enum[0], (intOrNone(enum[1]["value"]), trans(enum[1]["description"]))),
        exported_enum_listing["scriptEnums"]["LogicBatchMethod"]["values"].items(),
    )

    logictypes.update(exported_logictypes)
    slotlogictypes.update(exported_slotlogictypes)
    reagentmodes.update(exported_reagentmodes)
    batchmodes.update(exported_batchmodes)

    operations.update(exported_stationpedia["scriptCommands"].items())
    constants.update(exported_stationpedia["scriptConstants"].items())

    def map_enum_listing(
        item: tuple[str, EnumListing],
    ) -> Iterable[tuple[str, EnumEntry]]:
        name, listing = item
        if name == "_unnamed":
            name = ""
        else:
            name = f"{name}."
        return map(
            lambda entry: (f"{name}{entry[0]}", entry[1]), listing["values"].items()
        )

    exported_enums = map(
        lambda enum: (enum[0], (intOrNone(enum[1]["value"]), trans(enum[1]["description"]))),
        chain.from_iterable(
            map(map_enum_listing, exported_enum_listing["basicEnums"].items())
        ),
    )

    enums.update(exported_enums)

    help_patch_path = Path("data") / "help_patches.json"
    help_patches: HelpPatches = (
        None  # pyright: ignore[reportAssignmentType, reportUnusedVariable]
    )
    if help_patch_path.exists():
        with help_patch_path.open(mode="r") as f:
            help_patches = json.load(f)

    for name, help in help_patches["operations"].items():
        if name in operations and help:
            operations[name]["desc"] = help

    for name, help in help_patches["constants"].items():
        if name in constants and help:
            constants[name]["desc"] = help

    for name, help in help_patches["batchmodes"].items():
        if name in batchmodes and help:
            entry = batchmodes[name]
            batchmodes[name] = (entry[0], help)

    for name, help in help_patches["reagentmodes"].items():
        if name in reagentmodes and help:
            entry = reagentmodes[name]
            reagentmodes[name] = (entry[0], help)


    for name, help in help_patches["enums"].items():
        if name in enums and help:
            entry = enums[name]
            if entry[1]:
                print(f"(WARNING) enum {name} already has help!: {entry[1]}")
            enums[name] = (entry[0], help)

    # copy help form enums to logic types
    patches: list[tuple[str, tuple[int | None, str]]] = []
    for lt, lt_entry in logictypes.items():
        if f"LogicType.{lt}" in enums:
            enum_entry = enums[f"LogicType.{lt}"]
            if enum_entry[1] != lt_entry[1]:
                patches.append((lt, (lt_entry[0], enum_entry[1])))
    logictypes.update(patches)

    patches = []
    for lst, lst_entry in slotlogictypes.items():
        if f"LogicSlotType.{lst}" in enums:
            enum_entry = enums[f"LogicSlotType.{lst}"]
            if enum_entry[1] != lst_entry[1]:
                patches.append((lst, (lst_entry[0], enum_entry[1])))
    slotlogictypes.update(patches)

    op_help_path = Path("data") / "instructions_help.txt"
    with op_help_path.open(mode="w") as f:
        for key, val in sorted(operations.items()):
            help = val["desc"].replace("\r", "").replace("\n", "\\n")
            f.write(f"{key} {help}\n")

    stationpedia: dict[str, tuple[str, str | None]] = {}
    for page in exported_stationpedia["pages"]:
        if "PrefabName" not in page or not page["PrefabHash"]:
            continue
        stationpedia[str(page["PrefabHash"])] = (
            page["PrefabName"],
            trans(page["Title"]),
        )

    hashables_path = Path("data") / "stationpedia.txt"
    with hashables_path.open(mode="w") as f:
        for key, val in sorted(stationpedia.items(), key=lambda i: i[1][0]):
            name = val[0]
            desc = val[1] if val[1] is not None else ""
            f.write(
                "{} {} {}\n".format(
                    key, name, desc.replace("\r", "").replace("\n", "\\n")
                )
            )

    logic_types_path = Path("data") / "logictypes.txt"
    with logic_types_path.open(mode="w") as f:
        for t, (v, help) in sorted(logictypes.items()):
            f.write(f"{t} {v} {help.replace("\r", "").replace("\n", "\\n")}\n")
    slot_logic_types_path = Path("data") / "slotlogictypes.txt"
    with slot_logic_types_path.open(mode="w") as f:
        for t, (v, help) in sorted(slotlogictypes.items()):
            f.write(f"{t} {v} {help.replace("\r", "").replace("\n", "\\n")}\n")
    batch_modes_path = Path("data") / "batchmodes.txt"
    with batch_modes_path.open(mode="w") as f:
        for t, (v, help) in sorted(batchmodes.items()):
            f.write(f"{t} {v} {help.replace("\r", "").replace("\n", "\\n")}\n")
    reagent_modes_path = Path("data") / "reagentmodes.txt"
    with reagent_modes_path.open(mode="w") as f:
        for t, (v, help) in sorted(reagentmodes.items()):
            f.write(f"{t} {v} {help.replace("\r", "").replace("\n", "\\n")}\n")
    enums_path = Path("data") / "enums.txt"
    with enums_path.open(mode="w") as f:
        for name, (val, help) in sorted(enums.items()):
            f.write(f"{name} {val} {help.replace("\r", "").replace("\n", "\\n")}\n")

    constants_path = Path("data") / "constants.txt"
    with constants_path.open(mode="w") as f:
        for t, entry in sorted(constants.items()):
            v = entry["value"]
            help = entry["desc"]
            f.write(f"{t} {v} {help.replace("\r", "").replace("\n", "\\n")}\n")


if __name__ == "__main__":
    main()
