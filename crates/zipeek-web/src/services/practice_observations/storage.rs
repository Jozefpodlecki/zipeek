use web_sys::Storage;

use super::model::PracticeObservation;

pub(super) const OBSERVATION_PREFIX: &str = "zipeek.practice.observation.v1.";
const MAX_OBSERVATION_BYTES: usize = 64 * 1024;

pub(super) fn record(storage: &Storage, observation: &PracticeObservation) -> Result<(), String> {
    let encoded = observation.encoded()?;
    if encoded.len() > MAX_OBSERVATION_BYTES {
        return Err("Practice observation exceeds the storage limit".to_owned());
    }
    let key = format!("{OBSERVATION_PREFIX}{}", observation.step_id);
    match storage
        .get_item(&key)
        .map_err(|error| format!("Cannot read Practice observation: {error:?}"))?
    {
        Some(existing) if existing == encoded => Ok(()),
        Some(_) => Err("Conflicting previously saved Practice observation".to_owned()),
        None => storage
            .set_item(&key, &encoded)
            .map_err(|error| format!("Cannot save Practice observation: {error:?}")),
    }
}

fn observation_keys(storage: &Storage) -> Result<Vec<String>, String> {
    let count = storage
        .length()
        .map_err(|error| format!("Cannot enumerate Practice observations: {error:?}"))?;
    let mut keys = Vec::new();
    for index in 0..count {
        if let Some(key) = storage
            .key(index)
            .map_err(|error| format!("Cannot enumerate Practice observations: {error:?}"))?
            && key.starts_with(OBSERVATION_PREFIX)
        {
            keys.push(key);
        }
    }
    keys.sort_unstable();
    Ok(keys)
}

pub(super) fn read_inbox(storage: &Storage) -> Result<Vec<PracticeObservation>, String> {
    let mut observations = Vec::new();
    for key in observation_keys(storage)? {
        let raw = storage
            .get_item(&key)
            .map_err(|error| format!("Cannot read Practice observation: {error:?}"))?
            .ok_or("Practice observation disappeared during inbox enumeration")?;
        if raw.len() > MAX_OBSERVATION_BYTES {
            return Err("Practice observation exceeds the storage limit".to_owned());
        }
        let observation = PracticeObservation::decoded(&raw)?;
        if key != format!("{OBSERVATION_PREFIX}{}", observation.step_id) {
            return Err("Practice observation key does not match its identity".to_owned());
        }
        observations.push(observation);
    }
    observations.sort_by(|a, b| {
        a.finished_at_ms
            .cmp(&b.finished_at_ms)
            .then_with(|| a.step_id.cmp(&b.step_id))
    });
    Ok(observations)
}

pub(super) fn erase(storage: &Storage) -> Result<(), String> {
    for key in observation_keys(storage)? {
        storage
            .remove_item(&key)
            .map_err(|error| format!("Cannot erase Practice observation: {error:?}"))?;
    }
    Ok(())
}
