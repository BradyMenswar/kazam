use serde::{Deserialize, Serialize};

pub type Team = Vec<PokemonSet>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StatLine {
    #[serde(default)]
    pub hp: u16,
    #[serde(default)]
    pub atk: u16,
    #[serde(default)]
    pub def: u16,
    #[serde(default)]
    pub spa: u16,
    #[serde(default)]
    pub spd: u16,
    #[serde(default)]
    pub spe: u16,
}

impl StatLine {
    pub fn all(value: u16) -> Self {
        Self {
            hp: value,
            atk: value,
            def: value,
            spa: value,
            spd: value,
            spe: value,
        }
    }

    pub fn is_zero(&self) -> bool {
        self == &Self::default()
    }

    pub fn is_all(&self, value: u16) -> bool {
        self.hp == value
            && self.atk == value
            && self.def == value
            && self.spa == value
            && self.spd == value
            && self.spe == value
    }
}

pub fn default_ivs() -> StatLine {
    StatLine::all(31)
}

pub fn default_level() -> u8 {
    100
}

pub fn default_happiness() -> u16 {
    255
}

pub fn default_dynamax_level() -> u8 {
    10
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PokemonSet {
    #[serde(default)]
    pub name: String,
    pub species: String,
    #[serde(default)]
    pub gender: String,
    #[serde(default)]
    pub item: String,
    #[serde(default)]
    pub ability: String,
    #[serde(default)]
    pub evs: StatLine,
    #[serde(default)]
    pub nature: String,
    #[serde(default = "default_ivs")]
    pub ivs: StatLine,
    #[serde(default)]
    pub moves: Vec<String>,
    #[serde(default)]
    pub shiny: bool,
    #[serde(default = "default_level")]
    pub level: u8,
    #[serde(default = "default_happiness")]
    pub happiness: u16,
    #[serde(default)]
    pub pokeball: String,
    #[serde(default)]
    pub hidden_power_type: String,
    #[serde(default)]
    pub gigantamax: bool,
    #[serde(default = "default_dynamax_level")]
    pub dynamax_level: u8,
    #[serde(default)]
    pub tera_type: String,
}

impl Default for PokemonSet {
    fn default() -> Self {
        Self {
            name: String::new(),
            species: String::new(),
            gender: String::new(),
            item: String::new(),
            ability: String::new(),
            evs: StatLine::default(),
            nature: String::new(),
            ivs: default_ivs(),
            moves: Vec::new(),
            shiny: false,
            level: default_level(),
            happiness: default_happiness(),
            pokeball: String::new(),
            hidden_power_type: String::new(),
            gigantamax: false,
            dynamax_level: default_dynamax_level(),
            tera_type: String::new(),
        }
    }
}
