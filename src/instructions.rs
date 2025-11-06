use std::{fmt::Display, ops::Deref};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DataType {
    Number,
    Register,
    Device,
    LogicType,
    SlotLogicType,
    Name,
    BatchMode,
    ReagentMode,
}

#[derive(Debug)]
pub(crate) struct Union<'a>(pub(crate) &'a [DataType]);

#[derive(Debug)]
pub(crate) enum Param {
    Untagged(Union<'static>),
    Tagged(Union<'static>, &'static str),
}

impl Param {
    pub(crate) const fn with_tag(self, tag: &'static str) -> Self {
        match self {
            Self::Untagged(union) => Self::Tagged(union, tag),
            Self::Tagged(union, _) => Self::Tagged(union, tag),
        }
    }
}

impl Union<'static> {
    pub(crate) const fn as_tagged(self, tag: &'static str) -> Param {
        Param::Tagged(self, tag)
    }

    pub(crate) const fn as_untagged(self) -> Param {
        Param::Untagged(self)
    }
}

#[derive(Debug)]
pub(crate) struct InstructionSignature(pub(crate) &'static [Param]);

const REGISTER: Param = Union(&[DataType::Register]).as_untagged();
const DEVICE: Param = Union(&[DataType::Device]).as_untagged();
const VALUE: Param = Union(&[DataType::Register, DataType::Number]).as_untagged();
const REGISTER_ID: Param = Union(&[DataType::Register, DataType::Number]).as_tagged("deviceId");
const REGISTER_DEVICE: Param =
    Union(&[DataType::Register, DataType::Device]).as_tagged("registerOrDevice");
const DEVICE_ID: Param =
    Union(&[DataType::Device, DataType::Register, DataType::Number]).as_tagged("deviceOrId");

const LOGIC_TYPE: Param =
    Union(&[DataType::LogicType, DataType::Number, DataType::Register]).as_tagged("logicType");
const SLOT_LOGIC_TYPE: Param = Union(&[
    DataType::SlotLogicType,
    DataType::Number,
    DataType::Register,
])
.as_tagged("logicSlotType");
const BATCH_MODE: Param =
    Union(&[DataType::BatchMode, DataType::Number, DataType::Register]).as_tagged("batchMode");
const REAGENT_MODE: Param =
    Union(&[DataType::ReagentMode, DataType::Number, DataType::Register]).as_tagged("reagentMode");

const NAME: Param = Union(&[DataType::Name]).as_untagged();
const NUMBER: Param = Union(&[DataType::Number]).as_untagged();
#[allow(dead_code)]
const DEVICE_NAME: Param = Union(&[DataType::Register, DataType::Number]).as_tagged("deviceName");
#[allow(dead_code)]
const DEVICE_TYPE: Param = Union(&[DataType::Register, DataType::Number]).as_tagged("deviceType");
#[allow(dead_code)]
const INDEX: Param = Union(&[DataType::Register, DataType::Number]).as_tagged("index");
#[allow(dead_code)]
const ADDRESS: Param = Union(&[DataType::Register, DataType::Number]).as_tagged("memoryAddress");

include!(concat!(env!("OUT_DIR"), "/instructions.rs"));

include!(concat!(env!("OUT_DIR"), "/logictypes.rs"));

include!(concat!(env!("OUT_DIR"), "/modes.rs"));

include!(concat!(env!("OUT_DIR"), "/stationpedia.rs"));

include!(concat!(env!("OUT_DIR"), "/constants.rs"));

include!(concat!(env!("OUT_DIR"), "/enums.rs"));

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match *self {
            DataType::Number => "num",
            DataType::Register => "r?",
            DataType::Device => "d?",
            DataType::LogicType => "logicType",
            DataType::SlotLogicType => "slotType",
            DataType::Name => "name",
            DataType::BatchMode => "batchMode",
            DataType::ReagentMode => "reagentMode",
        };
        write!(f, "{}", val)
    }
}

impl Display for InstructionSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for parameter in self.0 {
            write!(f, " {parameter}")?;
        }
        Ok(())
    }
}

impl<'a> Display for Union<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let has_parens = self.0.len() != 1;
        if has_parens {
            write!(f, "(")?;
        }
        let mut first = true;
        for item in self.0.iter() {
            if !first {
                write!(f, "|")?;
            }
            first = false;
            item.fmt(f)?;
        }
        if has_parens {
            write!(f, ")")?;
        }
        Ok(())
    }
}

impl<'a> From<&'a [DataType]> for Union<'a> {
    fn from(value: &'a [DataType]) -> Self {
        Union(value)
    }
}

impl<'a> Union<'a> {
    pub(crate) fn match_type(&self, typ: DataType) -> bool {
        for x in self.0 {
            if *x == typ {
                return true;
            }
        }
        false
    }

    pub(crate) fn match_union(&self, types: &Union) -> bool {
        for typ in self.0 {
            for typ2 in types.0 {
                if typ == typ2 {
                    return true;
                }
            }
        }
        false
    }

    pub(crate) fn intersection(&self, other: &[DataType]) -> Vec<DataType> {
        self.0
            .iter()
            .filter(|x| other.contains(x))
            .map(Clone::clone)
            .collect()
    }
}

impl Deref for Param {
    type Target = Union<'static>;

    fn deref(&self) -> &Self::Target {
        match self {
            Param::Untagged(u) => u,
            Param::Tagged(u, _) => u,
        }
    }
}

impl Display for Param {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Param::Untagged(u) => std::fmt::Display::fmt(u, f),
            Param::Tagged(u, tag) => {
                let has_parens = u.0.len() != 1;
                if has_parens {
                    write!(f, "{}(", tag)?;
                }
                let mut first = true;
                for item in u.0.iter() {
                    if !first {
                        write!(f, "|")?;
                    }
                    first = false;
                    std::fmt::Display::fmt(item, f)?;
                }
                if has_parens {
                    write!(f, ")")?;
                }
                Ok(())
            }
        }
    }
}

pub(crate) fn logictype_candidates(text: &str) -> Vec<DataType> {
    let mut ret = Vec::with_capacity(3);

    if LOGIC_TYPES.contains(text) {
        ret.push(DataType::LogicType);
    }
    if SLOT_LOGIC_TYPES.contains(text) {
        ret.push(DataType::SlotLogicType);
    }
    if BATCH_MODES.contains(text) {
        ret.push(DataType::BatchMode);
    }
    if REAGENT_MODES.contains(text) {
        ret.push(DataType::ReagentMode);
    }

    ret
}
pub(crate) fn logictype_candidates_from_enum(val: &u16) -> Vec<DataType> {
    let mut ret = Vec::with_capacity(3);

    if LOGIC_TYPE_LOOKUP.contains_key(val) {
        ret.push(DataType::LogicType);
    }
    if SLOT_TYPE_LOOKUP.contains_key(val) {
        ret.push(DataType::SlotLogicType);
    }
    if BATCH_MODE_LOOKUP.contains_key(val) {
        ret.push(DataType::BatchMode);
    }
    if REAGENT_MODE_LOOKUP.contains_key(val) {
        ret.push(DataType::ReagentMode);
    }

    ret
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn matching_instructions() {
        for instruction in INSTRUCTIONS.keys() {
            println!("Is {instruction} in INSTRUCTION_DOCS?");
            assert!(INSTRUCTION_DOCS.contains_key(instruction));
        }
        for instruction in INSTRUCTION_DOCS.keys() {
            println!("Is {instruction} in INSTRUCTIONS?");
            assert!(INSTRUCTIONS.contains_key(instruction));
        }
    }

    #[test]
    fn matching_logic_types() {
        for logictype in LOGIC_TYPES.iter() {
            println!("Is {logictype} in LOGIC_TYPE_DOCS?");
            assert!(LOGIC_TYPE_DOCS.contains_key(logictype));
        }
        for logictype in LOGIC_TYPE_DOCS.keys() {
            println!("Is {logictype} in LOGIC_TYPES?");
            assert!(LOGIC_TYPES.contains(logictype));
        }
    }

    #[test]
    fn matching_slot_types() {
        for slottype in SLOT_LOGIC_TYPES.iter() {
            println!("Is {slottype} in SLOT_TYPE_DOCS?");
            assert!(SLOT_TYPE_DOCS.contains_key(slottype));
        }
        for slottype in SLOT_TYPE_DOCS.keys() {
            println!("Is {slottype} in SLOT_LOGIC_TYPES?");
            assert!(SLOT_LOGIC_TYPES.contains(slottype));
        }
    }
}
