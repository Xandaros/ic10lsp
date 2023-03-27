use std::fmt::Display;

use phf::{phf_map, phf_set};

#[derive(Clone, Copy, Debug, Eq)]
pub(crate) enum DataType {
    Number,
    Register,
    Device,
    LogicType,
    SlotLogicType,
    EitherLogicType,
}

impl PartialEq for DataType {
    fn eq(&self, other: &Self) -> bool {
        if core::mem::discriminant(self) == core::mem::discriminant(other) {
            return true;
        }
        use DataType::*;
        match self {
            LogicType | SlotLogicType => {
                return matches!(other, EitherLogicType);
            }
            EitherLogicType => {
                return matches!(other, LogicType | EitherLogicType);
            }
            _ => return false,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Union(pub(crate) &'static [DataType]);

#[derive(Debug)]
pub(crate) struct InstructionSignature(pub(crate) &'static [Union]);

const REGISTER: Union = Union(&[DataType::Register]);
const DEVICE: Union = Union(&[DataType::Device]);
const VALUE: Union = Union(&[DataType::Register, DataType::Number]);
const LOGIC_TYPE: Union = Union(&[DataType::LogicType]);
const SLOT_LOGIC_TYPE: Union = Union(&[DataType::SlotLogicType]);

pub(crate) const INSTRUCTIONS: phf::Map<&'static str, InstructionSignature> = phf_map! {
    "bdns" => InstructionSignature(&[DEVICE,VALUE]),
    "bdnsal" => InstructionSignature(&[DEVICE,VALUE]),
    "bdse" => InstructionSignature(&[DEVICE,VALUE]),
    "bdseal" => InstructionSignature(&[DEVICE,VALUE]),
    "brdns" => InstructionSignature(&[DEVICE,VALUE]),
    "brdse" => InstructionSignature(&[DEVICE,VALUE]),
    "l" => InstructionSignature(&[REGISTER,DEVICE,LOGIC_TYPE]),
    // "lb" => InstructionSignature(&[REGISTER,type,var,batchMode]),
    // "lr" => InstructionSignature(&[REGISTER,DEVICE,reagentMode,reagent]),
    "ls" => InstructionSignature(&[REGISTER,DEVICE,VALUE,SLOT_LOGIC_TYPE]),
    "s" => InstructionSignature(&[DEVICE,LOGIC_TYPE,VALUE]),
    // "sb" => InstructionSignature(&[type,var,REGISTER]),
    "bap" => InstructionSignature(&[VALUE,VALUE,VALUE,VALUE]),
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
    "bnaz" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "bnazal" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "bne" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "bneal" => InstructionSignature(&[VALUE,VALUE,VALUE]),
    "bnez" => InstructionSignature(&[VALUE,VALUE]),
    "bnezal" => InstructionSignature(&[VALUE,VALUE]),
    "brap" => InstructionSignature(&[VALUE,VALUE,VALUE,VALUE]),
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
    "sne" => InstructionSignature(&[REGISTER,VALUE,VALUE]),
    "snez" => InstructionSignature(&[REGISTER,VALUE]),
    "abs" => InstructionSignature(&[REGISTER,VALUE]),
    "acos" => InstructionSignature(&[REGISTER,VALUE]),
    "add" => InstructionSignature(&[REGISTER,VALUE,VALUE]),
    "asin" => InstructionSignature(&[REGISTER,VALUE]),
    "atan" => InstructionSignature(&[REGISTER,VALUE]),
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

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match *self {
            DataType::Number => "num",
            DataType::Register => "r?",
            DataType::Device => "d?",
            DataType::LogicType => "type",
            DataType::SlotLogicType => "slotType",
            DataType::EitherLogicType => "type|slotType",
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

impl Display for Union {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let has_parens = self.0.len() != 1
            || self
                .0
                .get(0)
                .map_or(false, |x| matches!(x, DataType::EitherLogicType));
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

impl Union {
    pub(crate) fn match_type(&self, typ: DataType) -> bool {
        for x in self.0 {
            if *x == typ {
                return true;
            }
        }
        return false;
    }
}
