//! Stat stages and related types

use kazam_protocol::Stat;

/// Stat stages (-6 to +6)
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StatStages {
    pub atk: i8,
    pub def: i8,
    pub spa: i8,
    pub spd: i8,
    pub spe: i8,
    pub accuracy: i8,
    pub evasion: i8,
}

impl StatStages {
    /// Create new stat stages (all at 0)
    pub fn new() -> Self {
        Self::default()
    }

    /// Get stage for a stat
    pub fn get(&self, stat: Stat) -> i8 {
        match stat {
            Stat::Atk => self.atk,
            Stat::Def => self.def,
            Stat::Spa => self.spa,
            Stat::Spd => self.spd,
            Stat::Spe => self.spe,
            Stat::Accuracy => self.accuracy,
            Stat::Evasion => self.evasion,
        }
    }

    /// Set stage for a stat (clamped to -6..+6)
    pub fn set(&mut self, stat: Stat, value: i8) {
        let clamped = value.clamp(-6, 6);
        match stat {
            Stat::Atk => self.atk = clamped,
            Stat::Def => self.def = clamped,
            Stat::Spa => self.spa = clamped,
            Stat::Spd => self.spd = clamped,
            Stat::Spe => self.spe = clamped,
            Stat::Accuracy => self.accuracy = clamped,
            Stat::Evasion => self.evasion = clamped,
        }
    }

    /// Apply a boost to a stat, returns actual change applied
    pub fn boost(&mut self, stat: Stat, amount: i8) -> i8 {
        let current = self.get(stat);
        let new_value = (current + amount).clamp(-6, 6);
        let actual_change = new_value - current;
        self.set(stat, new_value);
        actual_change
    }

    /// Apply an unboost (negative boost) to a stat, returns actual change applied
    pub fn unboost(&mut self, stat: Stat, amount: i8) -> i8 {
        self.boost(stat, -amount)
    }

    /// Reset all stages to 0
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Reset only positive stages to 0
    pub fn clear_positive(&mut self) {
        if self.atk > 0 {
            self.atk = 0;
        }
        if self.def > 0 {
            self.def = 0;
        }
        if self.spa > 0 {
            self.spa = 0;
        }
        if self.spd > 0 {
            self.spd = 0;
        }
        if self.spe > 0 {
            self.spe = 0;
        }
        if self.accuracy > 0 {
            self.accuracy = 0;
        }
        if self.evasion > 0 {
            self.evasion = 0;
        }
    }

    /// Reset only negative stages to 0
    pub fn clear_negative(&mut self) {
        if self.atk < 0 {
            self.atk = 0;
        }
        if self.def < 0 {
            self.def = 0;
        }
        if self.spa < 0 {
            self.spa = 0;
        }
        if self.spd < 0 {
            self.spd = 0;
        }
        if self.spe < 0 {
            self.spe = 0;
        }
        if self.accuracy < 0 {
            self.accuracy = 0;
        }
        if self.evasion < 0 {
            self.evasion = 0;
        }
    }

    /// Invert all stages (Topsy-Turvy)
    pub fn invert(&mut self) {
        self.atk = (-self.atk).clamp(-6, 6);
        self.def = (-self.def).clamp(-6, 6);
        self.spa = (-self.spa).clamp(-6, 6);
        self.spd = (-self.spd).clamp(-6, 6);
        self.spe = (-self.spe).clamp(-6, 6);
        self.accuracy = (-self.accuracy).clamp(-6, 6);
        self.evasion = (-self.evasion).clamp(-6, 6);
    }

    /// Copy boosts from another StatStages (Psych Up)
    pub fn copy_from(&mut self, other: &StatStages) {
        *self = other.clone();
    }

    /// Get the multiplier for a stat stage (for atk/def/spa/spd/spe)
    /// +1 = 1.5x, +2 = 2x, ..., +6 = 4x
    /// -1 = 0.67x, -2 = 0.5x, ..., -6 = 0.25x
    pub fn multiplier(stage: i8) -> f32 {
        let stage = stage.clamp(-6, 6);
        if stage >= 0 {
            (2 + stage as i32) as f32 / 2.0
        } else {
            2.0 / (2 - stage as i32) as f32
        }
    }

    /// Get the multiplier for accuracy/evasion stages (different formula)
    /// +1 = 1.33x, +2 = 1.67x, ..., +6 = 3x
    /// -1 = 0.75x, -2 = 0.6x, ..., -6 = 0.33x
    pub fn accuracy_multiplier(stage: i8) -> f32 {
        let stage = stage.clamp(-6, 6);
        if stage >= 0 {
            (3 + stage as i32) as f32 / 3.0
        } else {
            3.0 / (3 - stage as i32) as f32
        }
    }

    /// Check if all stats are at 0
    pub fn is_clear(&self) -> bool {
        self.atk == 0
            && self.def == 0
            && self.spa == 0
            && self.spd == 0
            && self.spe == 0
            && self.accuracy == 0
            && self.evasion == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stages_are_zero() {
        let stages = StatStages::new();
        assert_eq!(stages.atk, 0);
        assert_eq!(stages.def, 0);
        assert_eq!(stages.spa, 0);
        assert_eq!(stages.spd, 0);
        assert_eq!(stages.spe, 0);
        assert_eq!(stages.accuracy, 0);
        assert_eq!(stages.evasion, 0);
        assert!(stages.is_clear());
    }

    #[test]
    fn test_get_set() {
        let mut stages = StatStages::new();
        stages.set(Stat::Atk, 3);
        assert_eq!(stages.get(Stat::Atk), 3);

        stages.set(Stat::Spe, -2);
        assert_eq!(stages.get(Stat::Spe), -2);
    }

    #[test]
    fn test_set_clamps_to_bounds() {
        let mut stages = StatStages::new();
        stages.set(Stat::Atk, 10);
        assert_eq!(stages.atk, 6);

        stages.set(Stat::Def, -10);
        assert_eq!(stages.def, -6);
    }

    #[test]
    fn test_boost() {
        let mut stages = StatStages::new();

        // Normal boost
        let change = stages.boost(Stat::Atk, 2);
        assert_eq!(change, 2);
        assert_eq!(stages.atk, 2);

        // Boost that hits cap
        stages.atk = 5;
        let change = stages.boost(Stat::Atk, 3);
        assert_eq!(change, 1); // Only +1 actually applied
        assert_eq!(stages.atk, 6);

        // Boost when already at max
        let change = stages.boost(Stat::Atk, 1);
        assert_eq!(change, 0);
        assert_eq!(stages.atk, 6);
    }

    #[test]
    fn test_unboost() {
        let mut stages = StatStages::new();

        let change = stages.unboost(Stat::Def, 2);
        assert_eq!(change, -2);
        assert_eq!(stages.def, -2);

        // Unboost to minimum
        stages.def = -5;
        let change = stages.unboost(Stat::Def, 3);
        assert_eq!(change, -1);
        assert_eq!(stages.def, -6);
    }

    #[test]
    fn test_clear() {
        let mut stages = StatStages {
            atk: 3,
            def: -2,
            spa: 1,
            spd: -1,
            spe: 6,
            accuracy: 2,
            evasion: -3,
        };

        stages.clear();
        assert!(stages.is_clear());
    }

    #[test]
    fn test_clear_positive() {
        let mut stages = StatStages {
            atk: 3,
            def: -2,
            spa: 1,
            spd: -1,
            spe: 0,
            accuracy: 0,
            evasion: 0,
        };

        stages.clear_positive();
        assert_eq!(stages.atk, 0);
        assert_eq!(stages.def, -2); // Unchanged
        assert_eq!(stages.spa, 0);
        assert_eq!(stages.spd, -1); // Unchanged
    }

    #[test]
    fn test_clear_negative() {
        let mut stages = StatStages {
            atk: 3,
            def: -2,
            spa: 1,
            spd: -1,
            spe: 0,
            accuracy: 0,
            evasion: 0,
        };

        stages.clear_negative();
        assert_eq!(stages.atk, 3); // Unchanged
        assert_eq!(stages.def, 0);
        assert_eq!(stages.spa, 1); // Unchanged
        assert_eq!(stages.spd, 0);
    }

    #[test]
    fn test_invert() {
        let mut stages = StatStages {
            atk: 3,
            def: -2,
            spa: 0,
            spd: 6,
            spe: -6,
            accuracy: 1,
            evasion: -1,
        };

        stages.invert();
        assert_eq!(stages.atk, -3);
        assert_eq!(stages.def, 2);
        assert_eq!(stages.spa, 0);
        assert_eq!(stages.spd, -6);
        assert_eq!(stages.spe, 6);
        assert_eq!(stages.accuracy, -1);
        assert_eq!(stages.evasion, 1);
    }

    #[test]
    fn test_copy_from() {
        let source = StatStages {
            atk: 2,
            def: -1,
            spa: 3,
            spd: 0,
            spe: -2,
            accuracy: 1,
            evasion: -1,
        };

        let mut target = StatStages::new();
        target.copy_from(&source);

        assert_eq!(target, source);
    }

    #[test]
    fn test_stat_multiplier() {
        // Positive stages
        assert!((StatStages::multiplier(0) - 1.0).abs() < 0.001);
        assert!((StatStages::multiplier(1) - 1.5).abs() < 0.001);
        assert!((StatStages::multiplier(2) - 2.0).abs() < 0.001);
        assert!((StatStages::multiplier(3) - 2.5).abs() < 0.001);
        assert!((StatStages::multiplier(6) - 4.0).abs() < 0.001);

        // Negative stages
        assert!((StatStages::multiplier(-1) - 2.0 / 3.0).abs() < 0.001);
        assert!((StatStages::multiplier(-2) - 0.5).abs() < 0.001);
        assert!((StatStages::multiplier(-6) - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_accuracy_multiplier() {
        // Positive stages
        assert!((StatStages::accuracy_multiplier(0) - 1.0).abs() < 0.001);
        assert!((StatStages::accuracy_multiplier(1) - 4.0 / 3.0).abs() < 0.001);
        assert!((StatStages::accuracy_multiplier(6) - 3.0).abs() < 0.001);

        // Negative stages
        assert!((StatStages::accuracy_multiplier(-1) - 0.75).abs() < 0.001);
        assert!((StatStages::accuracy_multiplier(-6) - 1.0 / 3.0).abs() < 0.001);
    }
}
