use std::borrow::Cow;

pub use rosu_map::section::general::GameMode;

use crate::any::ModeDifficulty;

use super::beatmap::{Beatmap, Converted};

/// A way to specify a gamemode at compile-time.
///
/// Notably, this is implemented for the marker types [`Osu`], [`Taiko`],
/// [`Catch`], and [`Mania`].
///
/// [`Osu`]: crate::osu::Osu
/// [`Taiko`]: crate::taiko::Taiko
/// [`Catch`]: crate::catch::Catch
/// [`Mania`]: crate::mania::Mania
pub trait IGameMode: Sized {
    /// The resulting type of a difficulty calculation.
    type DifficultyAttributes;

    /// The resulting type of a strain calculation.
    type Strains;

    /// Attempt to convert a beatmap.
    ///
    /// In case [`ConvertStatus::Incompatible`] is returned, the map should
    /// **not** be modified.
    fn try_convert(map: &mut Cow<'_, Beatmap>) -> ConvertStatus;

    /// Perform a difficulty calculation for a [`Converted`] beatmap and
    /// process the final skill values.
    fn difficulty(
        difficulty: &ModeDifficulty,
        map: &Converted<'_, Self>,
    ) -> Self::DifficultyAttributes;

    /// Perform a difficulty calculation for a [`Converted`] beatmap without
    /// processing the final skill values.
    fn strains(difficulty: &ModeDifficulty, map: &Converted<'_, Self>) -> Self::Strains;
}

/// The status of a conversion through [`IGameMode::try_convert`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ConvertStatus {
    /// Conversion was not necessary.
    Noop,
    /// Conversion was successful.
    Done,
    /// Conversion was not possible.
    Incompatible,
}
