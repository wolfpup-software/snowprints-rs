// An i64 bit implementation
// SQL sortable without bit-shifting

const LOGICAL_VOLUME_BIT_LEN: i64 = 12;
const SEQUENCE_BIT_LEN: i64 = 10;
const LOGICAL_VOLUME_BIT_MASK: i64 = ((1 << LOGICAL_VOLUME_BIT_LEN) - 1) << SEQUENCE_BIT_LEN;
const MAX_LOGICAL_VOLUMES: i64 = i32::pow(2, LOGICAL_VOLUME_BIT_LEN as i32) as i64;
const MAX_SEQUENCES: i64 = i32::pow(2, SEQUENCE_BIT_LEN as u32) as i64;
const SEQUENCE_BIT_MASK: i64 = (1 << SEQUENCE_BIT_LEN) - 1;

const NEGATIVE_BIT_MASK: i64 = ~(1 << 63);

#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Snowprints {
    origin_time_duration: SystemTime,
    params: Params,
    state: State,
}

impl Snowprints {
    pub fn from(params: Params) -> Result<Snowprints, Errors> {
        if let Err(err) = check_params(&params) {
            return Err(err);
        }

        let origin_time_duration = UNIX_EPOCH + Duration::from_millis(params.origin_time_ms);

        let duration_ms = match SystemTime::now().duration_since(origin_time_duration) {
            Ok(duration) => duration.as_millis() as i64,
            _ => return Err(Errors::FailedToParseOriginSystemTime),
        };

        Ok(Snowprints {
            params: params,
            origin_time_duration: origin_time_duration,
            state: State {
                duration_ms: duration_ms,
                sequence: 0,
                logical_volume: 0,
                prev_logical_volume: 0,
            },
        })
    }

    pub fn create_id(&mut self) -> Result<i64, Errors> {
        let duration_ms =
            get_most_recent_duration_ms(self.origin_time_duration, self.state.duration_ms);

        match duration_ms > self.state.duration_ms {
            true => tick_logical_volume(&mut self.state, &self.params, duration_ms),
            _ => {
                if let Err(err) = tick_sequence(&mut self.state, &self.params) {
                    return Err(err);
                };
            }
        }

        Ok(compose(
            duration_ms,
            self.params.logical_volume_base + self.state.logical_volume,
            self.state.sequence,
        ))
    }

    pub fn get_timestamp(&self) -> i64 {
        get_most_recent_duration_ms(self.origin_time_duration, self.state.duration_ms)
    }

    pub fn get_bit_shifted_timestamp(&self, offset_ms: i64) -> i64 {
        let mut duration_ms =
            get_most_recent_duration_ms(self.origin_time_duration, self.state.duration_ms);

        duration_ms = match offset_ms < duration_ms {
            true => duration_ms - offset_ms,
            _ => 0,
        };

        compose(duration_ms, 0, 0)
    }
}

fn check_params(params: &Params) -> Result<(), Errors> {
    if params.logical_volume_length == 0 {
        return Err(Errors::LogicalVolumeModuloIsZero);
    }
    if MAX_LOGICAL_VOLUMES < (params.logical_volume_base + params.logical_volume_length) {
        return Err(Errors::ExceededAvailableLogicalVolumes);
    }

    Ok(())
}

fn get_most_recent_duration_ms(origin_time_duration: SystemTime, duration_ms: i64) -> i64 {
    if let Ok(duration) = SystemTime::now().duration_since(origin_time_duration) {
        let dur_ms = duration.as_millis() as i64;
        if duration_ms < dur_ms {
            return dur_ms;
        }
    }

    duration_ms
}

fn tick_logical_volume(state: &mut State, params: &Params, duration_ms: i64) {
    state.duration_ms = duration_ms;
    state.sequence = 0;
    state.prev_logical_volume = state.logical_volume;
    state.logical_volume = (state.logical_volume + 1) % params.logical_volume_length;
}

fn tick_sequence(state: &mut State, params: &Params) -> Result<(), Errors> {
    state.sequence += 1;
    if state.sequence < MAX_SEQUENCES {
        return Ok(());
    }

    state.sequence = 0;
    let next_logical_volume = (state.logical_volume + 1) % params.logical_volume_length;
    if state.prev_logical_volume != next_logical_volume {
        state.logical_volume = next_logical_volume;
        return Ok(());
    }

    Err(Errors::ExceededAvailableSequences)
}

pub fn compose64(ms_timestamp: i64, logical_volume: i64, ticket_id: i64) -> i64 {
    (ms_timestamp & NEGATIVE_BIT_MASK) << (LOGICAL_VOLUME_BIT_LEN + SEQUENCE_BIT_LEN)
        | (logical_volume & NEGATIVE_BIT_MASK) << SEQUENCE_BIT_LEN
        | (ticket_id & NEGATIVE_BIT_MASK)
}

pub fn decompose64(snowprint: i64) -> (i64, i64, i64) {
    let time = (snowprint & NEGATIVE_BIT_MASK) >> (LOGICAL_VOLUME_BIT_LEN + SEQUENCE_BIT_LEN);
    let logical_volume = (snowprint & LOGICAL_VOLUME_BIT_MASK) >> SEQUENCE_BIT_LEN;
    let ticket_id = snowprint & SEQUENCE_BIT_MASK;

    (time, logical_volume, ticket_id)
}
