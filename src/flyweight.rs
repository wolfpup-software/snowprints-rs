
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Params {
    pub logical_volume_base: u64,
    pub logical_volume_length: u64,
    pub origin_time_ms: u64,
}

#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Debug, Clone, Eq, PartialEq)]
struct State {
    pub duration_ms: u64,
    pub logical_volume: u64,
    pub prev_logical_volume: u64,
    pub sequence: u64,
}

#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Snowprints {
    origin_time_duration: SystemTime,
    params: Params,
    state: State,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Errors {
    ExceededAvailableLogicalVolumes,
    ExceededAvailableSequences,
    FailedToParseOriginSystemTime,
    LogicalVolumeModuloIsZero,
}

