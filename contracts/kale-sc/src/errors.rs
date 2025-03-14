use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Errors {
    HomesteadExists = 1,
    HomesteadMissing = 2,
    FarmBlockMissing = 3,
    FarmPaused = 4,
    FarmNotPaused = 5,
    PlantAmountTooLow = 6,
    ZeroCountTooLow = 7,
    PailExists = 8,
    PailMissing = 9,
    WorkMissing = 10,
    BlockMissing = 11,
    BlockInvalid = 12,
    HashInvalid = 13,
    HarvestNotReady = 14,
    GapCountTooLow = 15,
}
