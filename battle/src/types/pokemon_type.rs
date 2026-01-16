//! Pokemon type system and effectiveness chart

/// Pokemon types (18 types as of Gen 6+)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Type {
    Normal = 0,
    Fire = 1,
    Water = 2,
    Electric = 3,
    Grass = 4,
    Ice = 5,
    Fighting = 6,
    Poison = 7,
    Ground = 8,
    Flying = 9,
    Psychic = 10,
    Bug = 11,
    Rock = 12,
    Ghost = 13,
    Dragon = 14,
    Dark = 15,
    Steel = 16,
    Fairy = 17,
}

impl Type {
    /// All 18 Pokemon types
    pub const ALL: [Type; 18] = [
        Type::Normal,
        Type::Fire,
        Type::Water,
        Type::Electric,
        Type::Grass,
        Type::Ice,
        Type::Fighting,
        Type::Poison,
        Type::Ground,
        Type::Flying,
        Type::Psychic,
        Type::Bug,
        Type::Rock,
        Type::Ghost,
        Type::Dragon,
        Type::Dark,
        Type::Steel,
        Type::Fairy,
    ];

    /// Get all types as a slice
    pub fn all() -> &'static [Type] {
        &Self::ALL
    }

    /// Get type effectiveness against a single defending type
    pub fn effectiveness(&self, defender: Type) -> f32 {
        TYPE_CHART[*self as usize][defender as usize]
    }

    /// Get type effectiveness against multiple defending types (multiplied)
    pub fn effectiveness_multi(&self, defenders: &[Type]) -> f32 {
        defenders
            .iter()
            .map(|t| self.effectiveness(*t))
            .product()
    }

    /// Parse from protocol string (case-insensitive)
    pub fn from_protocol(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "normal" => Some(Type::Normal),
            "fire" => Some(Type::Fire),
            "water" => Some(Type::Water),
            "electric" => Some(Type::Electric),
            "grass" => Some(Type::Grass),
            "ice" => Some(Type::Ice),
            "fighting" => Some(Type::Fighting),
            "poison" => Some(Type::Poison),
            "ground" => Some(Type::Ground),
            "flying" => Some(Type::Flying),
            "psychic" => Some(Type::Psychic),
            "bug" => Some(Type::Bug),
            "rock" => Some(Type::Rock),
            "ghost" => Some(Type::Ghost),
            "dragon" => Some(Type::Dragon),
            "dark" => Some(Type::Dark),
            "steel" => Some(Type::Steel),
            "fairy" => Some(Type::Fairy),
            _ => None,
        }
    }

    /// Convert to canonical string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Type::Normal => "Normal",
            Type::Fire => "Fire",
            Type::Water => "Water",
            Type::Electric => "Electric",
            Type::Grass => "Grass",
            Type::Ice => "Ice",
            Type::Fighting => "Fighting",
            Type::Poison => "Poison",
            Type::Ground => "Ground",
            Type::Flying => "Flying",
            Type::Psychic => "Psychic",
            Type::Bug => "Bug",
            Type::Rock => "Rock",
            Type::Ghost => "Ghost",
            Type::Dragon => "Dragon",
            Type::Dark => "Dark",
            Type::Steel => "Steel",
            Type::Fairy => "Fairy",
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 18x18 type effectiveness chart
/// Row = attacking type, Column = defending type
/// Values: 0.0 = immune, 0.5 = not very effective, 1.0 = neutral, 2.0 = super effective
///
/// Order: Normal, Fire, Water, Electric, Grass, Ice, Fighting, Poison, Ground,
///        Flying, Psychic, Bug, Rock, Ghost, Dragon, Dark, Steel, Fairy
#[rustfmt::skip]
pub static TYPE_CHART: [[f32; 18]; 18] = [
    // Normal attacking
    [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.5, 0.0, 1.0, 1.0, 0.5, 1.0],
    // Fire attacking
    [1.0, 0.5, 0.5, 1.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 0.5, 1.0, 0.5, 1.0, 2.0, 1.0],
    // Water attacking
    [1.0, 2.0, 0.5, 1.0, 0.5, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 1.0, 2.0, 1.0, 0.5, 1.0, 1.0, 1.0],
    // Electric attacking
    [1.0, 1.0, 2.0, 0.5, 0.5, 1.0, 1.0, 1.0, 0.0, 2.0, 1.0, 1.0, 1.0, 1.0, 0.5, 1.0, 1.0, 1.0],
    // Grass attacking
    [1.0, 0.5, 2.0, 1.0, 0.5, 1.0, 1.0, 0.5, 2.0, 0.5, 1.0, 0.5, 2.0, 1.0, 0.5, 1.0, 0.5, 1.0],
    // Ice attacking
    [1.0, 0.5, 0.5, 1.0, 2.0, 0.5, 1.0, 1.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 0.5, 1.0],
    // Fighting attacking
    [2.0, 1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 0.5, 1.0, 0.5, 0.5, 0.5, 2.0, 0.0, 1.0, 2.0, 2.0, 0.5],
    // Poison attacking
    [1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 0.5, 0.5, 1.0, 1.0, 1.0, 0.5, 0.5, 1.0, 1.0, 0.0, 2.0],
    // Ground attacking
    [1.0, 2.0, 1.0, 2.0, 0.5, 1.0, 1.0, 2.0, 1.0, 0.0, 1.0, 0.5, 2.0, 1.0, 1.0, 1.0, 2.0, 1.0],
    // Flying attacking
    [1.0, 1.0, 1.0, 0.5, 2.0, 1.0, 2.0, 1.0, 1.0, 1.0, 1.0, 2.0, 0.5, 1.0, 1.0, 1.0, 0.5, 1.0],
    // Psychic attacking
    [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 1.0, 1.0, 0.5, 1.0, 1.0, 1.0, 1.0, 0.0, 0.5, 1.0],
    // Bug attacking
    [1.0, 0.5, 1.0, 1.0, 2.0, 1.0, 0.5, 0.5, 1.0, 0.5, 2.0, 1.0, 1.0, 0.5, 1.0, 2.0, 0.5, 0.5],
    // Rock attacking
    [1.0, 2.0, 1.0, 1.0, 1.0, 2.0, 0.5, 1.0, 0.5, 2.0, 1.0, 2.0, 1.0, 1.0, 1.0, 1.0, 0.5, 1.0],
    // Ghost attacking
    [0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 2.0, 1.0, 0.5, 1.0, 1.0],
    // Dragon attacking
    [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 0.5, 0.0],
    // Dark attacking
    [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.5, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 2.0, 1.0, 0.5, 1.0, 0.5],
    // Steel attacking
    [1.0, 0.5, 0.5, 0.5, 1.0, 2.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 1.0, 0.5, 2.0],
    // Fairy attacking
    [1.0, 0.5, 1.0, 1.0, 1.0, 1.0, 2.0, 0.5, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 0.5, 1.0],
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_effectiveness_super_effective() {
        assert_eq!(Type::Fire.effectiveness(Type::Grass), 2.0);
        assert_eq!(Type::Water.effectiveness(Type::Fire), 2.0);
        assert_eq!(Type::Electric.effectiveness(Type::Water), 2.0);
        assert_eq!(Type::Fighting.effectiveness(Type::Normal), 2.0);
    }

    #[test]
    fn test_type_effectiveness_not_very_effective() {
        assert_eq!(Type::Fire.effectiveness(Type::Water), 0.5);
        assert_eq!(Type::Grass.effectiveness(Type::Fire), 0.5);
        assert_eq!(Type::Electric.effectiveness(Type::Grass), 0.5);
    }

    #[test]
    fn test_type_effectiveness_immune() {
        assert_eq!(Type::Normal.effectiveness(Type::Ghost), 0.0);
        assert_eq!(Type::Ghost.effectiveness(Type::Normal), 0.0);
        assert_eq!(Type::Electric.effectiveness(Type::Ground), 0.0);
        assert_eq!(Type::Ground.effectiveness(Type::Flying), 0.0);
        assert_eq!(Type::Psychic.effectiveness(Type::Dark), 0.0);
        assert_eq!(Type::Dragon.effectiveness(Type::Fairy), 0.0);
    }

    #[test]
    fn test_type_effectiveness_multi() {
        // Fire vs Grass/Steel = 4x
        assert_eq!(Type::Fire.effectiveness_multi(&[Type::Grass, Type::Steel]), 4.0);
        // Fire vs Water/Rock = 0.25x
        assert_eq!(Type::Fire.effectiveness_multi(&[Type::Water, Type::Rock]), 0.25);
        // Electric vs Water/Flying = 4x
        assert_eq!(Type::Electric.effectiveness_multi(&[Type::Water, Type::Flying]), 4.0);
        // Ground vs Flying/Steel = 0x (immune)
        assert_eq!(Type::Ground.effectiveness_multi(&[Type::Flying, Type::Steel]), 0.0);
    }

    #[test]
    fn test_type_from_protocol() {
        assert_eq!(Type::from_protocol("Fire"), Some(Type::Fire));
        assert_eq!(Type::from_protocol("fire"), Some(Type::Fire));
        assert_eq!(Type::from_protocol("FIRE"), Some(Type::Fire));
        assert_eq!(Type::from_protocol("Psychic"), Some(Type::Psychic));
        assert_eq!(Type::from_protocol("unknown"), None);
    }

    #[test]
    fn test_type_as_str() {
        assert_eq!(Type::Fire.as_str(), "Fire");
        assert_eq!(Type::Psychic.as_str(), "Psychic");
        assert_eq!(Type::Normal.as_str(), "Normal");
    }

    #[test]
    fn test_all_types() {
        assert_eq!(Type::all().len(), 18);
        assert_eq!(Type::all()[0], Type::Normal);
        assert_eq!(Type::all()[17], Type::Fairy);
    }
}
