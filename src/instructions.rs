use std::fmt::Display;

use phf::{phf_map, phf_set};

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
pub(crate) struct InstructionSignature(pub(crate) &'static [Union<'static>]);

const REGISTER: Union = Union(&[DataType::Register]);
const DEVICE: Union = Union(&[DataType::Device]);
const VALUE: Union = Union(&[DataType::Register, DataType::Number]);
const LOGIC_TYPE: Union = Union(&[DataType::LogicType]);
const SLOT_LOGIC_TYPE: Union = Union(&[DataType::SlotLogicType]);
const BATCH_MODE: Union = Union(&[DataType::BatchMode, DataType::Number, DataType::Register]);
const REAGENT_MODE: Union = Union(&[DataType::ReagentMode, DataType::Number, DataType::Register]);

pub(crate) const INSTRUCTIONS: phf::Map<&'static str, InstructionSignature> = phf_map! {
    "alias" => InstructionSignature(&[Union(&[DataType::Name]), Union(&[DataType::Register, DataType::Device])]),
    "label" => InstructionSignature(&[Union(&[DataType::Name]), Union(&[DataType::Register, DataType::Device])]),
    "define" => InstructionSignature(&[Union(&[DataType::Name]), Union(&[DataType::Number])]),
    "bdns" => InstructionSignature(&[DEVICE,VALUE]),
    "bdnsal" => InstructionSignature(&[DEVICE,VALUE]),
    "bdse" => InstructionSignature(&[DEVICE,VALUE]),
    "bdseal" => InstructionSignature(&[DEVICE,VALUE]),
    "brdns" => InstructionSignature(&[DEVICE,VALUE]),
    "brdse" => InstructionSignature(&[DEVICE,VALUE]),
    "l" => InstructionSignature(&[REGISTER,DEVICE,LOGIC_TYPE]),
    "lb" => InstructionSignature(&[REGISTER,VALUE,LOGIC_TYPE,BATCH_MODE]),
    "lr" => InstructionSignature(&[REGISTER,DEVICE,REAGENT_MODE,VALUE]),
    "ls" => InstructionSignature(&[REGISTER,DEVICE,VALUE,SLOT_LOGIC_TYPE]),
    "s" => InstructionSignature(&[DEVICE,LOGIC_TYPE,VALUE]),
    "sb" => InstructionSignature(&[VALUE,LOGIC_TYPE,VALUE]),
    "bap" => InstructionSignature(&[VALUE,VALUE,VALUE,VALUE]),
    "bapal" => InstructionSignature(&[VALUE,VALUE,VALUE,VALUE]),
    "bapz" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "bapzal" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "beq" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "beqal" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "beqz" => InstructionSignature(&[VALUE,VALUE]),
    "beqzal" => InstructionSignature(&[VALUE,VALUE]),
    "bge" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "bgeal" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "bgez" => InstructionSignature(&[VALUE,VALUE]),
    "bgezal" => InstructionSignature(&[VALUE,VALUE]),
    "bgt" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "bgtal" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "bgtz" => InstructionSignature(&[VALUE,VALUE]),
    "bgtzal" => InstructionSignature(&[VALUE,VALUE]),
    "ble" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "bleal" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "blez" => InstructionSignature(&[VALUE,VALUE]),
    "blezal" => InstructionSignature(&[VALUE,VALUE]),
    "blt" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "bltal" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "bltz" => InstructionSignature(&[VALUE,VALUE]),
    "bltzal" => InstructionSignature(&[VALUE,VALUE]),
    "bna" => InstructionSignature(&[VALUE,VALUE,VALUE,VALUE]),
    "bnaal" => InstructionSignature(&[VALUE,VALUE,VALUE,VALUE]),
    "bnaz" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "bnazal" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "bne" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "bneal" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "bnez" => InstructionSignature(&[VALUE,VALUE]),
    "bnezal" => InstructionSignature(&[VALUE,VALUE]),
    "brap" => InstructionSignature(&[VALUE,VALUE,VALUE,VALUE]),
    "brapz" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "brnaz" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "breq" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "breqz" => InstructionSignature(&[VALUE,VALUE]),
    "brge" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "brgez" => InstructionSignature(&[VALUE,VALUE]),
    "brgt" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "brgtz" => InstructionSignature(&[VALUE,VALUE]),
    "brle" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "brlez" => InstructionSignature(&[VALUE,VALUE]),
    "brlt" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "brltz" => InstructionSignature(&[VALUE,VALUE]),
    "brna" => InstructionSignature(&[VALUE,VALUE,VALUE,VALUE]),
    "brne" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "brnez" => InstructionSignature(&[VALUE,VALUE]),
    "j" => InstructionSignature(&[VALUE]),
    "jal" => InstructionSignature(&[VALUE]),
    "jr" => InstructionSignature(&[VALUE]),
    "sap" => InstructionSignature(&[REGISTER,VALUE,VALUE,VALUE]),
    "sapz" => InstructionSignature(&[REGISTER,VALUE,VALUE]),
    "sdns" => InstructionSignature(&[REGISTER,DEVICE]),
    "sdse" => InstructionSignature(&[REGISTER,DEVICE]),
    "select" => InstructionSignature(&[REGISTER,VALUE,VALUE,VALUE]),
    "seq" => InstructionSignature(&[REGISTER,VALUE,VALUE]),
    "seqz" => InstructionSignature(&[REGISTER,VALUE]),
    "sge" => InstructionSignature(&[REGISTER,VALUE,VALUE]),
    "sgez" => InstructionSignature(&[REGISTER,VALUE]),
    "sgt" => InstructionSignature(&[REGISTER,VALUE,VALUE]),
    "sgtz" => InstructionSignature(&[REGISTER,VALUE]),
    "sle" => InstructionSignature(&[REGISTER,VALUE,VALUE]),
    "slez" => InstructionSignature(&[REGISTER,VALUE]),
    "slt" => InstructionSignature(&[REGISTER,VALUE,VALUE]),
    "sltz" => InstructionSignature(&[REGISTER,VALUE]),
    "sna" => InstructionSignature(&[REGISTER,VALUE,VALUE,VALUE]),
    "snaz" => InstructionSignature(&[REGISTER,VALUE,VALUE]),
    "sne" => InstructionSignature(&[REGISTER,VALUE,VALUE]),
    "snez" => InstructionSignature(&[REGISTER,VALUE]),
    "abs" => InstructionSignature(&[REGISTER,VALUE]),
    "acos" => InstructionSignature(&[REGISTER,VALUE]),
    "add" => InstructionSignature(&[REGISTER,VALUE,VALUE]),
    "asin" => InstructionSignature(&[REGISTER,VALUE]),
    "atan" => InstructionSignature(&[REGISTER,VALUE]),
    "atan2" => InstructionSignature(&[REGISTER,VALUE,VALUE]),
    "ceil" => InstructionSignature(&[REGISTER,VALUE]),
    "cos" => InstructionSignature(&[REGISTER,VALUE]),
    "div" => InstructionSignature(&[REGISTER,VALUE,VALUE]),
    "exp" => InstructionSignature(&[REGISTER,VALUE]),
    "floor" => InstructionSignature(&[REGISTER,VALUE]),
    "log" => InstructionSignature(&[REGISTER,VALUE]),
    "max" => InstructionSignature(&[REGISTER,VALUE,VALUE]),
    "min" => InstructionSignature(&[REGISTER,VALUE,VALUE]),
    "mod" => InstructionSignature(&[REGISTER,VALUE,VALUE]),
    "mul" => InstructionSignature(&[REGISTER,VALUE,VALUE]),
    "rand" => InstructionSignature(&[REGISTER]),
    "round" => InstructionSignature(&[REGISTER,VALUE]),
    "sin" => InstructionSignature(&[REGISTER,VALUE]),
    "sqrt" => InstructionSignature(&[REGISTER,VALUE]),
    "sub" => InstructionSignature(&[REGISTER,VALUE,VALUE]),
    "tan" => InstructionSignature(&[REGISTER,VALUE]),
    "trunc" => InstructionSignature(&[REGISTER,VALUE]),
    "and" => InstructionSignature(&[REGISTER,VALUE,VALUE]),
    "nor" => InstructionSignature(&[REGISTER,VALUE,VALUE]),
    "or" => InstructionSignature(&[REGISTER,VALUE,VALUE]),
    "xor" => InstructionSignature(&[REGISTER,VALUE,VALUE]),
    "peek" => InstructionSignature(&[REGISTER]),
    "pop" => InstructionSignature(&[REGISTER]),
    "push" => InstructionSignature(&[VALUE]),
    "hcf" => InstructionSignature(&[]),
    "move" => InstructionSignature(&[REGISTER,VALUE]),
    "sleep" => InstructionSignature(&[VALUE]),
    "yield" => InstructionSignature(&[]),
    "bnan" => InstructionSignature(&[VALUE, VALUE]),
    "brnan" => InstructionSignature(&[VALUE, VALUE]),
    "lbn" => InstructionSignature(&[REGISTER, VALUE, VALUE, LOGIC_TYPE, BATCH_MODE]),
    "lbns" => InstructionSignature(&[REGISTER, VALUE, VALUE, VALUE, SLOT_LOGIC_TYPE, BATCH_MODE]),
    "lbs" => InstructionSignature(&[REGISTER, VALUE, VALUE, SLOT_LOGIC_TYPE, BATCH_MODE]),
    "not" => InstructionSignature(&[REGISTER, VALUE]),
    "sbn" => InstructionSignature(&[VALUE, VALUE, LOGIC_TYPE, VALUE]),
    "sbs" => InstructionSignature(&[VALUE, VALUE, SLOT_LOGIC_TYPE, REGISTER]),
    "sla" => InstructionSignature(&[REGISTER, VALUE, VALUE]),
    "sll" => InstructionSignature(&[REGISTER, VALUE, VALUE]),
    "sra" => InstructionSignature(&[REGISTER, VALUE, VALUE]),
    "srl" => InstructionSignature(&[REGISTER, VALUE, VALUE]),
    "snan" => InstructionSignature(&[REGISTER, VALUE]),
    "snanz" => InstructionSignature(&[REGISTER, VALUE]),
    "ss" => InstructionSignature(&[DEVICE, VALUE, SLOT_LOGIC_TYPE, REGISTER]),
};

pub(crate) const LOGIC_TYPES: phf::Set<&'static str> = phf_set! {
    "Power",
    "Open",
    "Mode",
    "Error",
    "Lock",
    "Pressure",
    "Temperature",
    "PressureExternal",
    "PressureInternal",
    "Activate",
    "Charge",
    "Setting",
    "Reagents",
    "RatioOxygen",
    "RatioCarbonDioxide",
    "RatioNitrogen",
    "RatioPollutant",
    "RatioVolatiles",
    "RatioWater",
    "Horizontal",
    "Vertical",
    "SolarAngle",
    "Maximum",
    "Ratio",
    "PowerPotential",
    "PowerActual",
    "Quantity",
    "On",
    "ImportQuantity",
    "ImportSlotOccupant",
    "ExportQuantity",
    "ExportSlotOccupant",
    "RequiredPower",
    "HorizontalRatio",
    "VerticalRatio",
    "PowerRequired",
    "Idle",
    "Color",
    "ElevatorSpeed",
    "ElevatorLevel",
    "RecipeHash",
    "ExportSlotHash",
    "ImportSlotHash",
    "PlantHealth1",
    "PlantHealth2",
    "PlantHealth3",
    "PlantHealth4",
    "PlantGrowth1",
    "PlantGrowth2",
    "PlantGrowth3",
    "PlantGrowth4",
    "PlantEfficiency1",
    "PlantEfficiency2",
    "PlantEfficiency3",
    "PlantEfficiency4",
    "PlantHash1",
    "PlantHash2",
    "PlantHash3",
    "PlantHash4",
    "RequestHash",
    "CompletionRatio",
    "ClearMemory",
    "ExportCount",
    "ImportCount",
    "PowerGeneration",
    "TotalMoles",
    "Volume",
    "Plant",
    "Harvest",
    "Output",
    "PressureSetting",
    "TemperatureSetting",
    "TemperatureExternal",
    "Filtration",
    "AirRelease",
    "PositionX",
    "PositionY",
    "PositionZ",
    "VelocityMagnitude",
    "VelocityRelativeX",
    "VelocityRelativeY",
    "VelocityRelativeZ",
    "RatioNitrousOxide",
    "PrefabHash",
    "ForceWrite",
    "SignalStrength",
    "SignalID",
    "TargetX",
    "TargetY",
    "TargetZ",
    "SettingInput",
    "SettingOutput",
    "CurrentResearchPodType",
    "ManualResearchRequiredPod",
    "MineablesInVicinity",
    "MineablesInQueue",
    "NextWeatherEventTime",
    "Combustion",
    "Fuel",
    "ReturnFuelCost",
    "CollectableGoods",
    "Time",
    "Bpm",
    "EnvironmentEfficiency",
    "WorkingGasEfficiency",
    "PressureInput",
    "TemperatureInput",
    "RatioOxygenInput",
    "RatioCarbonDioxideInput",
    "RatioNitrogenInput",
    "RatioPollutantInput",
    "RatioVolatilesInput",
    "RatioWaterInput",
    "RatioNitrousOxideInput",
    "TotalMolesInput",
    "PressureInput2",
    "TemperatureInput2",
    "RatioOxygenInput2",
    "RatioCarbonDioxideInput2",
    "RatioNitrogenInput2",
    "RatioPollutantInput2",
    "RatioVolatilesInput2",
    "RatioWaterInput2",
    "RatioNitrousOxideInput2",
    "TotalMolesInput2",
    "PressureOutput",
    "TemperatureOutput",
    "RatioOxygenOutput",
    "RatioCarbonDioxideOutput",
    "RatioNitrogenOutput",
    "RatioPollutantOutput",
    "RatioVolatilesOutput",
    "RatioWaterOutput",
    "RatioNitrousOxideOutput",
    "TotalMolesOutput",
    "PressureOutput2",
    "TemperatureOutput2",
    "RatioOxygenOutput2",
    "RatioCarbonDioxideOutput2",
    "RatioNitrogenOutput2",
    "RatioPollutantOutput2",
    "RatioVolatilesOutput2",
    "RatioWaterOutput2",
    "RatioNitrousOxideOutput2",
    "TotalMolesOutput2",
    "CombustionInput",
    "CombustionInput2",
    "CombustionOutput",
    "CombustionOutput2",
    "OperationalTemperatureEfficiency",
    "TemperatureDifferentialEfficiency",
    "PressureEfficiency",
    "CombustionLimiter",
    "Throttle",
    "Rpm",
    "Stress",
    "InterrogationProgress",
    "TargetPadIndex",
    "SizeX",
    "SizeY",
    "SizeZ",
    "MinimumWattsToContact",
    "WattsReachingContact",
};

pub(crate) const SLOT_LOGIC_TYPES: phf::Set<&'static str> = phf_set! {
    "Occupied",
    "OccupantHash",
    "Quantity",
    "Damage",
    "Efficiency",
    "Health",
    "Growth",
    "Pressure",
    "Temperature",
    "Charge",
    "ChargeRatio",
    "Class",
    "PressureWaste",
    "PressureAir",
    "MaxQuantity",
    "Mature",
    "PrefabHash",
    "Seeding",
};

pub(crate) const BATCH_MODES: phf::Set<&'static str> = phf_set! {
    "Average",
    "Sum",
    "Minimum",
    "Maximum",
};

pub(crate) const BATCH_MODE_LOOKUP: phf::Map<u8, &'static str> = phf_map! {
    0u8 => "Average",
    1u8 => "Sum",
    2u8 => "Minimum",
    3u8 => "Maximum",
};

pub(crate) const REAGENT_MODES: phf::Set<&'static str> = phf_set! {
    "Contents",
    "Required",
    "Recipe",
};

pub(crate) const REAGENT_MODE_LOOKUP: phf::Map<u8, &'static str> = phf_map! {
    0u8 => "Contents",
    1u8 => "Required",
    2u8 => "Recipe",
};

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match *self {
            DataType::Number => "num",
            DataType::Register => "r?",
            DataType::Device => "d?",
            DataType::LogicType => "type",
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

// Taken directly from the game's rocketstation_Data/StreamingAssets/Language/english.xml
// with slight changes
pub(crate) const INSTRUCTION_DOCS: phf::Map<&'static str, &'static str> = phf_map! {
    "l" => "Loads device var to register.",
    "lb" => "Loads var from all output network devices with provided type hash using the provide batch mode. Average (0), Sum (1), Minimum (2), Maximum (3). Can use either the word, or the number.",
    "s" => "Stores register value to var on device.",
    "sb" => "Stores register value to var on all output network devices with provided type hash.",
    "ls" => "Loads slot var on device to register.",
    "lr" => "Loads reagent of device's reagentMode to register. Contents (0), Required (1), Recipe (2). Can use either the word, or the number.",
    "alias" => "Labels register or device reference with name, device references also affect what shows on the screws on the IC base.",
    "define" => "Creates a label that will be replaced throughout the program with the provided value.",
    "move" => "Register = provided num or register value.",
    "add" => "Register = a + b.",
    "sub" => "Register = a - b.",
    "sdse" => "Register = 1 if device is set, otherwise 0.",
    "sdns" => "Register = 1 if device is not set, otherwise 0",
    "slt" => "Register = 1 if a &lt; b, otherwise 0",
    "sgt" => "Register = 1 if a &gt; b, otherwise 0",
    "sle" => "Register = 1 if a &lt;= b, otherwise 0",
    "sge" => "Register = 1 if a &gt;= b, otherwise 0",
    "seq" => "Register = 1 if a == b, otherwise 0",
    "sne" => "Register = 1 if a != b, otherwise 0",
    "sap" => "Register = 1 if abs(a - b) &lt;= max(c * max(abs(a), abs(b)), float.epsilon * 8), otherwise 0",
    "sna" => "Register = 1 if abs(a - b) &gt; max(c * max(abs(a), abs(b)), float.epsilon * 8), otherwise 0",
    "sltz" => "Register = 1 if a &lt; 0, otherwise 0",
    "sgtz" => "Register = 1 if a &gt; 0, otherwise 0",
    "slez" => "Register = 1 if a &lt;= 0, otherwise 0",
    "sgez" => "Register = 1 if a &gt;= 0, otherwise 0",
    "seqz" => "Register = 1 if a == 0, otherwise 0",
    "snez" => "Register = 1 if a != 0, otherwise 0",
    "sapz" => "Register = 1 if abs(a) <= max(b * abs(a), float.epsilon * 8), otherwise 0",
    "snaz" => "Register = 1 if abs(a) > max(b * abs(a), float.epsilon * 8), otherwise 0",
    "and" => "Register = 1 if a and b not zero, otherwise 0",
    "or" => "Register = 1 if a and/or b not 0, otherwise 0",
    "xor" => "Register = 1 if either a or b not 0, otherwise 0",
    "nor" => "Register = 1 if a and b are 0, otherwise 0",
    "mul" => "Register = a * b",
    "div" => "Register = a / b",
    "mod" => "Register = a mod b (note: NOT a % b)",
    "j" => "Jump execution to line a",
    "bdse" => "Branch to line a if device d is set",
    "bdns" => "Branch to line a if device d isn't set",
    "blt" => "Branch to line c if a &lt; b",
    "bgt" => "Branch to line c if a &gt; b",
    "ble" => "Branch to line c if a &lt;= b",
    "bge" => "Branch to line c if a &gt;= b",
    "beq" => "Branch to line c if a == b",
    "bne" => "Branch to line c if a != b",
    "bap" => "Branch to line d if abs(a - b) &lt;= max(c * max(abs(a), abs(b)), float.epsilon * 8)",
    "bna" => "Branch to line d if abs(a - b) &gt; max(c * max(abs(a), abs(b)), float.epsilon * 8)",
    "bltz" => "Branch to line b if a &lt; 0",
    "bgez" => "Branch to line b if a &gt;= 0",
    "blez" => "Branch to line b if a &lt;= 0",
    "bgtz" => "Branch to line b if a &gt; 0",
    "beqz" => "Branch to line b if a == 0",
    "bnez" => "Branch to line b if a != 0",
    "bapz" => "Branch to line c if abs(a) &lt;= float.epsilon * 8",
    "bnaz" => "Branch to line c if abs(a) &gt; float.epsilon * 8",
    "jr" => "Relative jump to line a",
    "brdse" => "Relative jump to line a if device is set",
    "brdns" => "Relative jump to line a if device is not set",
    "brlt" => "Relative jump to line c if a &lt; b",
    "brgt" => "Relative jump to line c if a &gt; b",
    "brle" => "Relative jump to line c if a &lt;= b",
    "brge" => "Relative jump to line c if a &gt;= b",
    "breq" => "Relative branch to line c if a == b",
    "brne" => "Relative branch to line c if a != b",
    "brap" => "Relative branch to line d if abs(a - b) &lt;= max(c * max(abs(a), abs(b)), float.epsilon * 8)",
    "brna" => "Relative branch to line d if abs(a - b) &gt; max(c * max(abs(a), abs(b)), float.epsilon * 8)",
    "brltz" => "Relative branch to line b if a &lt; 0",
    "brgez" => "Relative branch to line b if a &gt;= 0",
    "brlez" => "Relative branch to line b if a &lt;= 0",
    "brgtz" => "Relative branch to line b if a &gt; 0",
    "breqz" => "Relative branch to line b if a == 0",
    "brnez" => "Relative branch to line b if a != 0",
    "brapz" => "Relative branch to line c if abs(a) &lt;= float.epsilon * 8",
    "brnaz" => "Relative branch to line c if abs(a) &gt; float.epsilon * 8",
    "jal" => "Jump execution to line a and store next line number in ra",
    "bdseal" => "Jump execution to line a and store next line number if device is set",
    "bdnsal" => "Jump execution to line a and store next line number if device is not set",
    "bltal" => "Branch to line c if a &lt; b and store next line number in ra",
    "bgtal" => "Branch to line c if a &gt; b and store next line number in ra",
    "bleal" => "Branch to line c if a &lt;= b and store next line number in ra",
    "bgeal" => "Branch to line c if a &gt;= b and store next line number in ra",
    "beqal" => "Branch to line c if a == b and store next line number in ra",
    "bneal" => "Branch to line c if a != b and store next line number in ra",
    "bapal" => "Branch to line d if abs(a - b) &lt;= max(c * max(abs(a), abs(b)), float.epsilon * 8) and store next line number in ra",
    "bnaal" => "Branch to line d if abs(a - b) &lt;= max(c * max(abs(a), abs(b)), float.epsilon * 8) and store next line number in ra",
    "bltzal" => "Branch to line b if a &lt; 0 and store next line number in ra",
    "bgezal" => "Branch to line b if a &gt;= 0 and store next line number in ra",
    "blezal" => "Branch to line b if a &lt;= 0 and store next line number in ra",
    "bgtzal" => "Branch to line b if a &gt; 0 and store next line number in ra",
    "beqzal" => "Branch to line b if a == 0 and store next line number in ra",
    "bnezal" => "Branch to line b if a != 0 and store next line number in ra",
    "bapzal" => "Branch to line c if abs(a) &lt;= float.epsilon * 8",
    "bnazal" => "Branch to line c if abs(a) &gt; float.epsilon * 8",
    "sqrt" => "Register = square root of a",
    "round" => "Register = a rounded to nearest integer",
    "trunc" => "Register = a with fractional part removed",
    "ceil" => "Register = smallest integer greater than a",
    "floor" => "Register = largest integer less than a",
    "max" => "Register = max of a or b",
    "min" => "Register = min of a or b",
    "abs" => "Register = the absolute value of a",
    "log" => "Register = log(a)",
    "exp" => "Register = exp(a)",
    "rand" => "Register = a random value x with 0 &lt;= x &lt; 1",
    "yield" => "Pauses execution for 1 tick",
    "label" => "DEPRECATED - Use alias instead",
    "peek" => "Register = the value at the top of the stack",
    "push" => "Pushes the value of a to the stack at sp and increments sp",
    "pop" => "Register = the value at the top of the stack and decrements sp",
    "hcf" => "Halt and catch fire",
    "select" => "Register = b if a is non-zero, otherwise c",
    "sleep" => "Pauses execution on the IC for a seconds",
    "sin" => "Returns the sine of the specified angle (radians)",
    "cos" => "Returns the cosine of the specified angle (radians)",
    "tan" => "Returns the tan of the specified angle (radians) ",
    "asin" => "Returns the angle (radians) whos sine is the specified value",
    "acos" => "Returns the angle (radians) whos cosine is the specified value",
    "atan" => "Returns the angle (radians) whos tan is the specified value",
    "atan2" => "Returns the angle (radians) whose tangent is the quotient of two specified values: a (y) and b (x)",
};

pub(crate) const LOGIC_TYPE_DOCS: phf::Map<&'static str, &'static str> = phf_map! {
    "Power" => "Can be read to return if the device is correctly powered or not, set via the power system, return 1 if powered and 0 if not",
    "Open" => "1 if device is open, otherwise 0",
    "Mode" => "Integer for mode state, different devices will have different mode states available to them",
    "Error" => "1 if device is in error state, otherwise 0",
    "Lock" => "1 if device is locked, otherwise 0, can be set in most devices and prevents the user from access the values",
    "Pressure" => "The current pressure reading of the device",
    "Temperature" => "The current temperature reading of the device",
    "PressureInput" => "The current pressure reading of the device's Input Network",
    "TemperatureInput" => "The current temperature reading of the device's Input Network",
    "PressureInput2" => "The current pressure reading of the device's Input2 Network",
    "TemperatureInput2" => "The current temperature reading of the device's Input2 Network",
    "PressureOutput" => "The current pressure reading of the device's Output Network",
    "TargetPadIndex" => "The index of the trader landing pad on this devices data network that it will try to call a trader in to land",
    "InterrogationProgress" => "Progress of this sattellite dish's interrogation of its current target, as a ratio from 0-1",
    "SizeX" => "Size on the X(right) axis of the object in largeGrids (a largeGrid is 2meters)",
    "SizeY" => "Size on the Y(Up) axis of the object in largeGrids (a largeGrid is 2meters)",
    "SizeZ" => "Size on the Z(Forward) axis of the object in largeGrids (a largeGrid is 2meters)",
    "MinimumWattsToContact" => "Minimum required amount of watts from the dish hitting the target trader contact to start interrogating the contact",
    "WattsReachingContact" => "The amount of watts actually hitting the contact. This is effected by the power of the dish and how far off-axis the dish is from the contact vector",
    "TemperatureOutput" => "The current temperature reading of the device's Output Network",
    "PressureOutput2" => "The current pressure reading of the device's Output2 Network",
    "TemperatureOutput2" => "The current temperature reading of the device's Output2 Network",
    "PressureExternal" => "Setting for external pressure safety, in KPa",
    "PressureInternal" => "Setting for internal pressure safety, in KPa",
    "Activate" => "1 if device is activated (usually means running), otherwise 0",
    "Charge" => "The current charge the device has",
    "Setting" => "A variable setting that can be read or written, depending on the device",
    "Reagents" => "Total number of reagents recorded by the device",
    "RatioOxygen" => "The ratio of oxygen in device atmosphere",
    "RatioCarbonDioxide" => "The ratio of carbon dioxide in device atmosphere",
    "RatioNitrogen" => "The ratio of nitrogen in device atmosphere",
    "RatioPollutant" => "The ratio of pollutant in device atmosphere",
    "RatioVolatiles" => "The ratio of volatiles in device atmosphere",
    "RatioWater" => "The ratio of water in device atmosphere",
    "RatioNitrousOxide" => "The ratio of nitrous oxide in device atmosphere",
    "Combustion" => "The assess atmosphere is on fire. Returns 1 if atmosphere is on fire, 0 if not.",
    "RatioOxygenInput" => "The ratio of oxygen in device's input network",
    "RatioCarbonDioxideInput" => "The ratio of carbon dioxide in device's input network",
    "RatioNitrogenInput" => "The ratio of nitrogen in device's input network",
    "RatioPollutantInput" => "The ratio of pollutant in device's input network",
    "RatioVolatilesInput" => "The ratio of volatiles in device's input network",
    "RatioWaterInput" => "The ratio of water in device's input network",
    "RatioNitrousOxideInput" => "The ratio of nitrous oxide in device's input network",
    "CombustionInput" => "The assess atmosphere is on fire. Returns 1 if device's input network is on fire, 0 if not.",
    "RatioOxygenInput2" => "The ratio of oxygen in device's Input2 network",
    "RatioCarbonDioxideInput2" => "The ratio of carbon dioxide in device's Input2 network",
    "RatioNitrogenInput2" => "The ratio of nitrogen in device's Input2 network",
    "RatioPollutantInput2" => "The ratio of pollutant in device's Input2 network",
    "RatioVolatilesInput2" => "The ratio of volatiles in device's Input2 network",
    "RatioWaterInput2" => "The ratio of water in device's Input2 network",
    "RatioNitrousOxideInput2" => "The ratio of nitrous oxide in device's Input2 network",
    "CombustionInput2" => "The assess atmosphere is on fire. Returns 1 if device's Input2 network is on fire, 0 if not.",
    "RatioOxygenOutput" => "The ratio of oxygen in device's Output network",
    "RatioCarbonDioxideOutput" => "The ratio of carbon dioxide in device's Output network",
    "RatioNitrogenOutput" => "The ratio of nitrogen in device's Output network",
    "RatioPollutantOutput" => "The ratio of pollutant in device's Output network",
    "RatioVolatilesOutput" => "The ratio of volatiles in device's Output network",
    "RatioWaterOutput" => "The ratio of water in device's Output network",
    "RatioNitrousOxideOutput" => "The ratio of nitrous oxide in device's Output network",
    "CombustionOutput" => "The assess atmosphere is on fire. Returns 1 if device's Output network is on fire, 0 if not.",
    "RatioOxygenOutput2" => "The ratio of oxygen in device's Output2 network",
    "RatioCarbonDioxideOutput2" => "The ratio of carbon dioxide in device's Output2 network",
    "RatioNitrogenOutput2" => "The ratio of nitrogen in device's Output2 network",
    "RatioPollutantOutput2" => "The ratio of pollutant in device's Output2 network",
    "RatioVolatilesOutput2" => "The ratio of volatiles in device's Output2 network",
    "RatioWaterOutput2" => "The ratio of water in device's Output2 network",
    "RatioNitrousOxideOutput2" => "The ratio of nitrous oxide in device's Output2 network",
    "CombustionOutput2" => "The assess atmosphere is on fire. Returns 1 if device's Output2 network is on fire, 0 if not.",
    "Horizontal" => "Horizontal setting of the device",
    "Vertical" => "Vertical setting of the device",
    "SolarAngle" => "Solar angle of the device",
    "Maximum" => "Maximum setting of the device",
    "Ratio" => "Context specific value depending on device, 0 to 1 based ratio",
    "PowerPotential" => "How much energy the device or network potentially provides",
    "PowerActual" => "How much energy the device or network is actually using",
    "Quantity" => "Total quantity on the device",
    "On" => "The current state of the device, 0 for off, 1 for on",
    "ImportQuantity" => "Total quantity of items imported by the device",
    "ImportSlotOccupant" => "DEPRECATED",
    "ExportQuantity" => "Total quantity of items exported by the device",
    "ExportSlotOccupant" => "DEPRECATED",
    "RequiredPower" => "Idle operating power quantity, does not necessarily include extra demand power",
    "HorizontalRatio" => "Radio of horizontal setting for device",
    "VerticalRatio" => "Radio of vertical setting for device",
    "PowerRequired" => "Power requested from the device and/or network",
    "Idle" => "Returns 1 if the device is currently idle, otherwise 0",
    "Color" =>
"Whether driven by concerns for clarity, safety or simple aesthetics, Stationeers have access to a small rainbow of colors for their constructions. These are the color setting for devices, represented as an integer.

0: Blue
1: Grey
2: Green
3: Orange
4: Red
5: Yellow
6: White
7: Black
8: Brown
9: Khaki
10: Pink
11: Purple

It is an unwavering universal law that anything higher than 11 will be purple. The ODA is powerless to change this. Similarly, anything lower than 0 will be Blue.",
    "ElevatorSpeed" => "Current speed of the elevator",
    "ElevatorLevel" => "Level the elevator is currently at",
    "RecipeHash" => "Current hash of the recipe the device is set to produce",
    "ExportSlotHash" => "DEPRECATED",
    "ImportSlotHash" => "DEPRECATED",
    "PlantHealth1" => "DEPRECATED",
    "PlantHealth2" => "DEPRECATED",
    "PlantHealth3" => "DEPRECATED",
    "PlantHealth4" => "DEPRECATED",
    "PlantGrowth1" => "DEPRECATED",
    "PlantGrowth2" => "DEPRECATED",
    "PlantGrowth3" => "DEPRECATED",
    "PlantGrowth4" => "DEPRECATED",
    "PlantEfficiency1" => "DEPRECATED",
    "PlantEfficiency2" => "DEPRECATED",
    "PlantEfficiency3" => "DEPRECATED",
    "PlantEfficiency4" => "DEPRECATED",
    "PlantHash1" => "DEPRECATED",
    "PlantHash2" => "DEPRECATED",
    "PlantHash3" => "DEPRECATED",
    "PlantHash4" => "DEPRECATED",
    "CompletionRatio" => "How complete the current production is for this device, between 0 and 1",
    "ClearMemory" => "When set to 1, clears the counter memory (e.g. ExportCount). Will set itself back to 0 when actioned",
    "ExportCount" => "How many items exported since last ClearMemory",
    "ImportCount" => "How many items imported since last ClearMemory",
    "PowerGeneration" => "Returns how much power is being generated",
    "TotalMoles" => "Returns the total moles of the device",
    "TotalMolesInput" => "Returns the total moles of the device's Input Network",
    "TotalMolesInput2" => "Returns the total moles of the device's Input2 Network",
    "TotalMolesOutput" => "Returns the total moles of the device's Output Network",
    "TotalMolesOutput2" => "Returns the total moles of the device's Output2 Network",
    "Volume" => "Returns the device atmosphere volume",
    "Plant" => "Performs the planting action for any plant based machinery",
    "Harvest" => "Performs the harvesting action for any plant based machinery",
    "Output" => "The output operation for a sort handling device, such as a stacker or sorter, when in logic mode the device will only action one repetition when set zero or above and then back to -1 and await further instructions",
    "PressureSetting" => "The current setting for the internal pressure of the object (e.g. the Hardsuit Air release), in KPa",
    "TemperatureSetting" => "The current setting for the internal temperature of the object (e.g. the Hardsuit A/C)",
    "TemperatureExternal" => "The temperature of the outside of the device, usually the world atmosphere surrounding it",
    "Filtration" => "The current state of the filtration system, for example Filtration = 1 for a Hardsuit sets filtration to On",
    "AirRelease" => "The current state of the air release system, for example AirRelease = 1 for a Hardsuit sets Air Release to On",
    "PositionX" => "The current position in X dimension in world coordinates",
    "PositionY" => "The current position in Y dimension in world coordinates",
    "PositionZ" => "The current position in Z dimension in world coordinates",
    "VelocityMagnitude" => "The current magnitude of the velocity vector",
    "VelocityRelativeX" => "The current velocity X relative to the forward vector of this",
    "VelocityRelativeY" => "The current velocity Y relative to the forward vector of this",
    "VelocityRelativeZ" => "The current velocity Z relative to the forward vector of this",
    "PrefabHash" => "The hash of the structure",
    "ForceWrite" => "Forces Logic Writer devices to rewrite value",
    "SignalStrength" => "Returns the degree offset of the strongest contact",
    "SignalID" => "Returns the contact ID of the strongest signal from this Satellite",
    "OperationalTemperatureEfficiency" => "How the input pipe's temperature effects the machines efficiency",
    "TemperatureDifferentialEfficiency" => "How the difference between the input pipe and waste pipe temperatures effect the machines efficiency",
    "CombustionLimiter" => "Retards the rate of combustion inside the machine (range: 0-100), with 0 being the slowest rate of combustion and 100 being the fastest",
    "Throttle" => "Increases the rate at which the machine works (range: 0-100)",
    "Rpm" => "The number of revolutions per minute that the device's spinning mechanism is doing",
    "Stress" => "Machines get stressed when working hard. When Stress reaches 100 the machine will automatically shut down",
    "PressureEfficiency" => "How the pressure of the input pipe and waste pipe effect the machines efficiency",
    "ReturnFuelCost" => "Gets the fuel remaining in your rocket's fuel tank.",
    "Fuel" => "Gets the cost of fuel to return the rocket to your current world.",
    "CollectableGoods" => "Returns 0 or 1 based on whether or not the current rocket system has the ability to collect resources on the planet.",
    "RequestHash" => "When set to the unique identifier, requests an item of the provided type from the device",
    "ManualResearchRequiredPod" => "Sets the pod type to search for a certain pod when breaking down a pods.",
    "Bpm" => "Bpm",
    "SettingInput" => "Undocumented.",
    "SettingOutput" => "Undocumented.",
    "EnvironmentEfficiency" => "The Environment Efficiency reported by the machine, as a float between 0 and 1",
    "NextWeatherEventTime" => "Returns in seconds when the next weather event is inbound.",
    "TargetX" => "The target position in X dimension in world coordinates",
    "TargetY" => "The target position in Y dimension in world coordinates",
    "TargetZ" => "The target position in Z dimension in world coordinates",
    "Time" => "Time",
    "WorkingGasEfficiency" => "The Working Gas Efficiency reported by the machine, as a float between 0 and 1",
    "CurrentResearchPodType" => "Undocumented.",
    "MineablesInVicinity" => "Returns the amount of potential mineables within an extended area around AIMEe.",
    "MineablesInQueue" => "Returns the amount of mineables AIMEe has queued up to mine.",
};

pub(crate) const SLOT_TYPE_DOCS: phf::Map<&'static str, &'static str> = phf_map! {
    "Occupied" => "Returns 0 when slot is not occupied, 1 when it is",
    "OccupantHash" => "Returns the has of the current occupant, the unique identifier of the thing",
    "Quantity" => "Returns the current quantity, such as stack size, of the item in the slot",
    "Damage" => "Returns the damage state of the item in the slot",
    "Efficiency" => "Returns the growth efficiency of the plant in the slot",
    "Mature" => "Returns 1 if the plant in this slot is mature, 0 when it isn't",
    "Seeding" => "Whether a plant is seeding (ready to harvest seeds from). Returns 1 if seeding or 0 if not.",
    "Health" => "Returns the health of the plant in the slot",
    "Growth" => "Returns the current growth state of the plant in the slot",
    "Pressure" => "Returns pressure of the slot occupants internal atmosphere",
    "Temperature" => "Returns temperature of the slot occupants internal atmosphere",
    "Charge" => "Returns current energy charge the slot occupant is holding",
    "ChargeRatio" => "Returns current energy charge the slot occupant is holding as a ratio between 0 and 1 of its maximum",
    "Class" => "Returns integer representing the class of object",
    "PressureWaste" => "Returns pressure in the waste tank of the jetpack in this slot",
    "PressureAir" => "Returns pressure in the air tank of the jetpack in this slot",
    "MaxQuantity" => "Returns the max stack size of the item in the slot",
    "PrefabHash" => "Returns the hash of the structure in the slot",
};

pub(crate) const BATCH_MODE_DOCS: phf::Map<&'static str, &'static str> = phf_map! {
    "Average" => "Average of all read values",
    "Sum" => "All read values added together",
    "Minimum" => "Lowest of all read values",
    "Maximum" => "Highest of all read values",
};

include!(concat!(env!("OUT_DIR"), "/stationpedia.rs"));

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
