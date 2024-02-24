use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    fmt::Display,
    str::FromStr,
    sync::{Arc, OnceLock},
};

use anyhow::anyhow;
use regex::Regex;

use crate::dbus_api;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Display for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}x{}", self.width, self.height))
    }
}

impl PartialOrd for Resolution {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Resolution {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.width.cmp(&other.width) {
            Ordering::Equal => self.height.cmp(&other.height),
            ord => ord,
        }
    }
}

impl FromStr for Resolution {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static RESOLUTION_RE: OnceLock<Regex> = OnceLock::new();
        let re = RESOLUTION_RE.get_or_init(|| Regex::new(r"^(\d+)x(\d+)$").unwrap());
        let (_, [width, height]) = re
            .captures_iter(s)
            .map(|c| c.extract())
            .next()
            .ok_or(anyhow!("wrong resolution format"))?;
        let width = width
            .parse()
            .map_err(|_| anyhow!("could not parse resolution width"))?;
        let height = height
            .parse()
            .map_err(|_| anyhow!("could not parse resolution height"))?;
        Ok(Resolution { width, height })
    }
}

/// Structure that coresponds to a single real mode with its frequency rounded
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RoundedMode {
    res: Resolution,
    frequency: u32,
}

impl Display for RoundedMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}@{}", self.res, self.frequency))
    }
}

pub struct ModeDb {
    modes: Arc<[RoundedMode]>,
    resolutions: Arc<[Resolution]>,
    res_to_freqs: HashMap<Resolution, Arc<[u32]>>,
    mode_to_id: HashMap<RoundedMode, u32>,
    id_to_mode: HashMap<u32, RoundedMode>,
}

impl ModeDb {
    pub fn new(modes: &[dbus_api::Mode]) -> Self {
        type ModeAndRoundedFreq<'a> = (Vec<&'a dbus_api::Mode>, HashSet<u32>);
        let mut res_to_mode_and_freq: HashMap<Resolution, ModeAndRoundedFreq<'_>> = HashMap::new();

        // Group every mode by resolution and find set of rounded frequencies for given resolution
        for mode in modes {
            let rounded_freq: u32 = mode.frequency.round() as u32;
            res_to_mode_and_freq
                .entry(Resolution {
                    width: mode.width,
                    height: mode.height,
                })
                .and_modify(|(res_m, res_f)| {
                    res_m.push(mode);
                    res_f.insert(rounded_freq);
                })
                .or_insert_with(|| {
                    let mut freqs = HashSet::new();
                    freqs.insert(rounded_freq);
                    (vec![mode], freqs)
                });
        }

        // Find mode that has the closest frequency to rounded one to link back to it;
        let mut rounded_modes = vec![];
        let mut resolutions = vec![];
        let mut mode_to_id = HashMap::new();
        let mut id_to_mode = HashMap::new();
        let mut res_to_freqs = HashMap::new();
        for (res, (modes, freqs)) in res_to_mode_and_freq {
            resolutions.push(res.clone());

            let mut freqs: Vec<_> = freqs.iter().cloned().collect();
            freqs.sort_by(|l, r| r.cmp(l));
            for &frequency in freqs.iter() {
                let rounded_mode = RoundedMode {
                    res: res.clone(),
                    frequency,
                };

                // Mode frequency needs to be within 1Hz to be considered "close"
                let mut min_diff = 1f64;
                let mut mode_id = 0u32;
                // Searching for id of mode that will represent given rounded mode
                for mode in &modes {
                    let diff = (frequency as f64 - mode.frequency).abs();
                    if diff < min_diff {
                        min_diff = diff;
                        mode_id = mode.id;
                        // Additionally link every real mode to rounded one
                        id_to_mode.insert(mode.id, rounded_mode.clone());
                    }
                }

                rounded_modes.push(rounded_mode.clone());
                mode_to_id.insert(rounded_mode, mode_id);
            }
            res_to_freqs.insert(res.clone(), freqs.into());
        }
        rounded_modes.sort_by(|l, r| r.cmp(l));
        resolutions.sort_by(|l, r| r.cmp(l));

        ModeDb {
            modes: rounded_modes.into(),
            resolutions: resolutions.into(),
            mode_to_id,
            id_to_mode,
            res_to_freqs,
        }
    }

    pub fn get_modes(&self) -> Arc<[RoundedMode]> {
        self.modes.clone()
    }

    pub fn get_resolutions(&self) -> Arc<[Resolution]> {
        self.resolutions.clone()
    }

    pub fn get_resolutions_with_frequencies(&self) -> Arc<[ResolutionFrequencies]> {
        let mut result = vec![];
        for res in self.get_resolutions().iter() {
            let freqs = self
                .get_res_frequencies(res)
                .expect("Known resolutions should have matching frequencies");
            result.push(ResolutionFrequencies {
                res: res.clone(),
                freqs,
            });
        }
        result.into()
    }

    pub fn get_res_frequencies(&self, res: &Resolution) -> Option<Arc<[u32]>> {
        self.res_to_freqs.get(res).cloned()
    }

    /// Returns an id of real Mode
    pub fn get_id(&self, mode: &RoundedMode) -> u32 {
        *self
            .mode_to_id
            .get(mode)
            .expect("RoundedMode should be valid")
    }

    pub fn get_mode(&self, res: Resolution, frequency: u32) -> Option<&RoundedMode> {
        self.mode_to_id
            .get(&RoundedMode { res, frequency })
            .map(|id| {
                self.id_to_mode
                    .get(id)
                    .expect("Id was found, should have coresponding mode")
            })
    }

    /// Returns RoundedMode given an id of real Mode
    pub fn get_mode_by_id(&self, mode_id: u32) -> Option<&RoundedMode> {
        self.id_to_mode.get(&mode_id)
    }

    pub fn get_modes_by_ids(&self, mode_ids: &[u32]) -> Arc<[RoundedMode]> {
        let mut unique_modes = HashSet::new();
        for &id in mode_ids {
            if let Some(mode) = self.get_mode_by_id(id) {
                unique_modes.insert(mode);
            }
        }
        let unique_modes: Vec<_> = unique_modes.into_iter().cloned().collect();
        unique_modes.into()
    }
}

#[derive(Debug, Clone)]
pub struct ResolutionFrequencies {
    res: Resolution,
    freqs: Arc<[u32]>,
}

impl PartialEq for ResolutionFrequencies {
    fn eq(&self, other: &Self) -> bool {
        self.res.eq(&other.res)
    }
}
impl Eq for ResolutionFrequencies {}

impl PartialOrd for ResolutionFrequencies {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ResolutionFrequencies {
    fn cmp(&self, other: &Self) -> Ordering {
        self.res.cmp(&other.res)
    }
}

impl Display for ResolutionFrequencies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}, freqs: {:?}", self.res, self.freqs))
    }
}

pub fn group_modes_by_res(modes: &[RoundedMode]) -> Arc<[ResolutionFrequencies]> {
    let mut res_freqs = HashMap::new();
    for mode in modes {
        res_freqs
            .entry(mode.res.clone())
            .and_modify(|f: &mut Vec<_>| f.push(mode.frequency))
            .or_insert_with(|| vec![mode.frequency]);
    }
    let mut result: Vec<_> = res_freqs
        .into_iter()
        .map(|(res, mut freqs)| {
            freqs.sort_by(|l, r| r.cmp(l));
            ResolutionFrequencies {
                res,
                freqs: freqs.into(),
            }
        })
        .collect();
    result.sort_by(|l, r| r.cmp(l));
    result.into()
}
