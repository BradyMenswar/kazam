//! Type matchup helpers for decision making

use crate::types::Type;

/// Check if defender is weak (>1x effectiveness) to any of the attacking types
pub fn is_weak_to_any(defender_types: &[Type], attacking_types: &[Type]) -> bool {
    attacking_types
        .iter()
        .any(|t| t.effectiveness_multi(defender_types) > 1.0)
}

/// Check if defender resists (<1x effectiveness) all of the attacking types
pub fn resists_all(defender_types: &[Type], attacking_types: &[Type]) -> bool {
    if attacking_types.is_empty() {
        return false;
    }
    attacking_types
        .iter()
        .all(|t| t.effectiveness_multi(defender_types) < 1.0)
}

/// Check if defender is immune (0x effectiveness) to a type
pub fn is_immune_to(defender_types: &[Type], attacking_type: Type) -> bool {
    attacking_type.effectiveness_multi(defender_types) == 0.0
}

/// Get all types that are super effective against the defender
pub fn weaknesses(defender_types: &[Type]) -> Vec<Type> {
    Type::all()
        .iter()
        .copied()
        .filter(|t| t.effectiveness_multi(defender_types) > 1.0)
        .collect()
}

/// Get all types that the defender resists (0 < effectiveness < 1)
pub fn resistances(defender_types: &[Type]) -> Vec<Type> {
    Type::all()
        .iter()
        .copied()
        .filter(|t| {
            let eff = t.effectiveness_multi(defender_types);
            eff > 0.0 && eff < 1.0
        })
        .collect()
}

/// Get all types that the defender is immune to
pub fn immunities(defender_types: &[Type]) -> Vec<Type> {
    Type::all()
        .iter()
        .copied()
        .filter(|t| t.effectiveness_multi(defender_types) == 0.0)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_weak_to_any() {
        let water = vec![Type::Water];
        let attacking = vec![Type::Electric, Type::Grass];
        assert!(is_weak_to_any(&water, &attacking));

        let neutral = vec![Type::Fire, Type::Ice];
        assert!(!is_weak_to_any(&water, &neutral));
    }

    #[test]
    fn test_resists_all() {
        // Steel resists Normal, Flying, Rock, Bug, Steel, Grass, Psychic, Ice, Dragon, Fairy
        let steel = vec![Type::Steel];
        let resisted = vec![Type::Normal, Type::Ice, Type::Fairy];
        assert!(resists_all(&steel, &resisted));

        let not_resisted = vec![Type::Fire, Type::Ice];
        assert!(!resists_all(&steel, &not_resisted));
    }

    #[test]
    fn test_is_immune_to() {
        let ghost = vec![Type::Ghost];
        assert!(is_immune_to(&ghost, Type::Normal));
        assert!(is_immune_to(&ghost, Type::Fighting));
        assert!(!is_immune_to(&ghost, Type::Dark));

        let normal = vec![Type::Normal];
        assert!(is_immune_to(&normal, Type::Ghost));

        let ground = vec![Type::Ground];
        assert!(is_immune_to(&ground, Type::Electric));
    }

    #[test]
    fn test_weaknesses() {
        // Steel type is weak to Fire, Fighting, Ground
        let steel = vec![Type::Steel];
        let weak = weaknesses(&steel);
        assert!(weak.contains(&Type::Fire));
        assert!(weak.contains(&Type::Fighting));
        assert!(weak.contains(&Type::Ground));
        assert_eq!(weak.len(), 3);
    }

    #[test]
    fn test_weaknesses_dual_type() {
        // Water/Ground (Swampert) is only weak to Grass (4x)
        let swampert = vec![Type::Water, Type::Ground];
        let weak = weaknesses(&swampert);
        assert_eq!(weak, vec![Type::Grass]);
    }

    #[test]
    fn test_resistances() {
        // Steel resists many types
        let steel = vec![Type::Steel];
        let resists = resistances(&steel);
        assert!(resists.contains(&Type::Normal));
        assert!(resists.contains(&Type::Ice));
        assert!(resists.contains(&Type::Fairy));
        // Fire, Fighting, Ground are weaknesses, not resistances
        assert!(!resists.contains(&Type::Fire));
    }

    #[test]
    fn test_immunities() {
        // Ghost is immune to Normal and Fighting
        let ghost = vec![Type::Ghost];
        let immune = immunities(&ghost);
        assert!(immune.contains(&Type::Normal));
        assert!(immune.contains(&Type::Fighting));
        assert_eq!(immune.len(), 2);
    }
}
