use std::{io, path::Path, str::FromStr};

use rosu_map::{
    section::{events::BreakPeriod, general::GameMode, hit_objects::hit_samples::HitSoundType},
    LATEST_FORMAT_VERSION,
};

pub use self::{
    attributes::{BeatmapAttributes, BeatmapAttributesBuilder, HitWindows},
    converted::Converted,
    decode::{BeatmapState, ParseBeatmapError},
};

use super::{
    control_point::{
        difficulty_point_at, effect_point_at, timing_point_at, DifficultyPoint, EffectPoint,
        TimingPoint,
    },
    hit_object::HitObject,
    mode::IGameMode,
};

mod attributes;
mod bpm;
mod converted;
mod decode;

/// All beatmap data that is relevant for difficulty and performance
/// calculation.
#[derive(Clone, Debug, PartialEq)]
pub struct Beatmap {
    pub version: i32,

    // General
    pub stack_leniency: f32,
    pub mode: GameMode,

    // Difficulty
    pub ar: f32,
    pub cs: f32,
    pub hp: f32,
    pub od: f32,
    pub slider_multiplier: f64,
    pub slider_tick_rate: f64,

    // Events
    pub breaks: Vec<BreakPeriod>,

    // TimingPoints
    pub timing_points: Vec<TimingPoint>,
    pub difficulty_points: Vec<DifficultyPoint>,
    pub effect_points: Vec<EffectPoint>,

    // HitObjects
    pub hit_objects: Vec<HitObject>,
    pub hit_sounds: Vec<HitSoundType>,
}

impl Beatmap {
    /// Parse a [`Beatmap`] by providing a path to a `.osu` file.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        rosu_map::from_path(path)
    }

    /// Parse a [`Beatmap`] by providing the content of a `.osu` file as a
    /// slice of bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, io::Error> {
        rosu_map::from_bytes(bytes)
    }

    /// Finds the [`TimingPoint`] that is active at the given time.
    pub(crate) fn timing_point_at(&self, time: f64) -> Option<&TimingPoint> {
        timing_point_at(&self.timing_points, time)
    }

    /// Finds the [`DifficultyPoint`] that is active at the given time.
    pub(crate) fn difficulty_point_at(&self, time: f64) -> Option<&DifficultyPoint> {
        difficulty_point_at(&self.difficulty_points, time)
    }

    /// Finds the [`EffectPoint`] that is active at the given time.
    pub(crate) fn effect_point_at(&self, time: f64) -> Option<&EffectPoint> {
        effect_point_at(&self.effect_points, time)
    }

    /// Sum up the duration of all breaks (in milliseconds).
    pub fn total_break_time(&self) -> f64 {
        self.breaks.iter().map(BreakPeriod::duration).sum()
    }

    /// Attempt to convert a [`&Beatmap`] to the specified mode.
    ///
    /// If the conversion is incompatible, `None` is returned.
    ///
    /// [`&Beatmap`]: Beatmap
    pub fn try_as_converted<M: IGameMode>(&self) -> Option<Converted<'_, M>> {
        Converted::try_from_ref(self)
    }

    /// Convert a [`&Beatmap`] to the specified mode.
    ///
    /// # Panics
    ///
    /// Panics if the conversion is incompatible.
    ///
    /// [`&Beatmap`]: Beatmap
    pub fn unchecked_as_converted<M: IGameMode>(&self) -> Converted<'_, M> {
        Converted::unchecked_from_ref(self)
    }

    /// Attempt to convert a [`Beatmap`] to the specified mode.
    ///
    /// If the conversion is incompatible the [`Beatmap`] will be returned
    /// unchanged as `Err`.
    #[allow(clippy::result_large_err)]
    pub fn try_into_converted<'a, M: IGameMode>(self) -> Result<Converted<'a, M>, Self> {
        Converted::try_from_owned(self)
    }

    /// Convert a [`Beatmap`] to the specified mode.
    ///
    /// # Panics
    ///
    /// Panics if the conversion is incompatible.
    pub fn unchecked_into_converted<'a, M: IGameMode>(self) -> Converted<'a, M> {
        Converted::unchecked_from_owned(self)
    }

    /// Returns a [`BeatmapAttributesBuilder`] to calculate modified beatmap
    /// attributes.
    pub fn attributes(&self) -> BeatmapAttributesBuilder {
        self.into()
    }

    /// The beats per minute of the map.
    pub fn bpm(&self) -> f64 {
        bpm::bpm(self.hit_objects.last(), &self.timing_points)
    }
}

impl FromStr for Beatmap {
    type Err = io::Error;

    /// Parse a [`Beatmap`] by providing the content of a `.osu` file as a
    /// string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        rosu_map::from_str(s)
    }
}

const DEFAULT_SLIDER_LENIENCY: f32 = 0.7;

impl Default for Beatmap {
    fn default() -> Self {
        Self {
            version: LATEST_FORMAT_VERSION,
            stack_leniency: DEFAULT_SLIDER_LENIENCY,
            mode: GameMode::default(),
            ar: 5.0,
            cs: 5.0,
            hp: 5.0,
            od: 5.0,
            slider_multiplier: 1.4,
            slider_tick_rate: 1.0,
            breaks: Vec::default(),
            timing_points: Vec::default(),
            difficulty_points: Vec::default(),
            effect_points: Vec::default(),
            hit_objects: Vec::default(),
            hit_sounds: Vec::default(),
        }
    }
}

macro_rules! define_beat_len_fn {
    ( $fn_name:ident, $clamp:literal) => {
        #[allow(unused)]
        pub(crate) fn $fn_name(map: &Beatmap, beat_len: f64, time: f64) -> f64 {
            let slider_velocity = map
                .difficulty_point_at(time)
                .map_or(DifficultyPoint::DEFAULT_SLIDER_VELOCITY, |point| {
                    point.slider_velocity
                });

            let slider_velocity_as_beat_len = -100.0 / slider_velocity;

            let bpm_mult = if slider_velocity_as_beat_len < 0.0 {
                (((-slider_velocity_as_beat_len) as f32).clamp(10.0, $clamp) / 100.0) as f64
            } else {
                1.0
            };

            beat_len * bpm_mult
        }
    };
}

define_beat_len_fn!(get_precision_adjusted_beat_len_taiko_mania, 10_000.0);
define_beat_len_fn!(get_precision_adjusted_beat_len_osu_catch, 1000.0);
