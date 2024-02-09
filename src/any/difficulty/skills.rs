pub fn strain_decay(ms: f64, strain_decay_base: f64) -> f64 {
    strain_decay_base.powf(ms / 1000.0)
}

/// Wrapper around a difficulty skill that carries a list of all difficulty
/// objects.
pub struct Skill<'a, S: ISkill> {
    pub inner: S,
    pub diff_objects: &'a S::DifficultyObjects<'a>,
}

impl<'a, S: ISkill> Skill<'a, S> {
    pub const fn new(skill: S, diff_objects: &'a S::DifficultyObjects<'a>) -> Self {
        Self {
            inner: skill,
            diff_objects,
        }
    }
}

/// Trait required for [`Skill`].
pub trait ISkill {
    type DifficultyObjects<'a>: ?Sized;
}

#[derive(Clone, Default)]
pub struct StrainSkill {
    pub curr_section_peak: f64,
    pub curr_section_end: f64,
    pub strain_peaks: Vec<f64>, // TODO: default capacity
}

impl StrainSkill {
    pub const DECAY_WEIGHT: f64 = 0.9;
    pub const SECTION_LEN: f64 = 400.0;

    pub fn save_curr_peak(&mut self) {
        self.strain_peaks.push(self.curr_section_peak);
    }

    pub fn start_new_section_from(&mut self, initial_strain: f64) {
        self.curr_section_peak = initial_strain;
    }

    pub fn get_curr_strain_peaks(self) -> Vec<f64> {
        let mut strain_peaks = self.strain_peaks;
        strain_peaks.push(self.curr_section_peak);

        strain_peaks
    }

    pub fn difficulty_value(self, decay_weight: f64) -> f64 {
        let mut difficulty = 0.0;
        let mut weight = 1.0;

        let mut peaks = self.get_curr_strain_peaks();
        peaks.retain(|&strain| strain > 0.0);
        peaks.sort_by(|a, b| b.total_cmp(a));

        for strain in peaks {
            difficulty += strain * weight;
            weight *= decay_weight;
        }

        difficulty
    }
}

#[derive(Clone, Default)]
pub struct StrainDecaySkill {
    pub inner: StrainSkill,
    pub curr_strain: f64,
}

impl StrainDecaySkill {
    pub const DECAY_WEIGHT: f64 = StrainSkill::DECAY_WEIGHT;
    pub const SECTION_LEN: f64 = StrainSkill::SECTION_LEN;

    pub fn save_curr_peak(&mut self) {
        self.inner.save_curr_peak();
    }

    pub fn start_new_section_from(&mut self, initial_strain: f64) {
        self.inner.start_new_section_from(initial_strain);
    }

    pub fn get_curr_strain_peaks(self) -> Vec<f64> {
        self.inner.get_curr_strain_peaks()
    }

    pub fn difficulty_value(self, decay_weight: f64) -> f64 {
        self.inner.difficulty_value(decay_weight)
    }
}
