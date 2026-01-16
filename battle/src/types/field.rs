//! Global field state

use super::conditions::{Terrain, Weather};

/// Global field state affecting all Pokemon
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FieldState {
    /// Current weather condition
    pub weather: Option<Weather>,

    /// Current terrain
    pub terrain: Option<Terrain>,

    /// Trick Room active (slower Pokemon move first)
    pub trick_room: bool,

    /// Magic Room active (items suppressed)
    pub magic_room: bool,

    /// Wonder Room active (Def/SpD swapped)
    pub wonder_room: bool,

    /// Gravity active (Flying immunity removed, accuracy boosted)
    pub gravity: bool,

    /// Mud Sport active (Electric moves weakened) - older gens
    pub mud_sport: bool,

    /// Water Sport active (Fire moves weakened) - older gens
    pub water_sport: bool,

    /// Ion Deluge active (Normal moves become Electric)
    pub ion_deluge: bool,

    /// Fairy Lock active (no switching)
    pub fairy_lock: bool,
}

impl FieldState {
    /// Create a new empty field state
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset all field conditions
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Set weather from a protocol field start message
    pub fn set_weather_from_protocol(&mut self, condition: &str) {
        self.weather = Weather::from_protocol(condition);
    }

    /// Clear weather
    pub fn clear_weather(&mut self) {
        self.weather = None;
    }

    /// Set terrain from a protocol field start message
    pub fn set_terrain_from_protocol(&mut self, condition: &str) {
        self.terrain = Terrain::from_protocol(condition);
    }

    /// Clear terrain
    pub fn clear_terrain(&mut self) {
        self.terrain = None;
    }

    /// Apply a field start condition from protocol
    pub fn apply_field_start(&mut self, condition: &str) {
        // Strip common prefixes
        let clean = condition
            .strip_prefix("move: ")
            .unwrap_or(condition);

        // Normalize
        let normalized = clean.to_lowercase().replace([' ', '-'], "");

        match normalized.as_str() {
            // Weather (handled separately usually, but just in case)
            "sunnyday" | "raindance" | "sandstorm" | "hail" | "snow" | "desolateland"
            | "primordialsea" | "deltastream" => {
                self.weather = Weather::from_protocol(condition);
            }

            // Terrain
            "electricterrain" | "grassyterrain" | "mistyterrain" | "psychicterrain" => {
                self.terrain = Terrain::from_protocol(condition);
            }

            // Rooms
            "trickroom" => self.trick_room = true,
            "magicroom" => self.magic_room = true,
            "wonderroom" => self.wonder_room = true,

            // Other
            "gravity" => self.gravity = true,
            "mudsport" => self.mud_sport = true,
            "watersport" => self.water_sport = true,
            "iondeluge" => self.ion_deluge = true,
            "fairylock" => self.fairy_lock = true,

            _ => {}
        }
    }

    /// Apply a field end condition from protocol
    pub fn apply_field_end(&mut self, condition: &str) {
        // Strip common prefixes
        let clean = condition
            .strip_prefix("move: ")
            .unwrap_or(condition);

        // Normalize
        let normalized = clean.to_lowercase().replace([' ', '-'], "");

        match normalized.as_str() {
            // Terrain
            "electricterrain" | "grassyterrain" | "mistyterrain" | "psychicterrain" => {
                self.terrain = None;
            }

            // Rooms
            "trickroom" => self.trick_room = false,
            "magicroom" => self.magic_room = false,
            "wonderroom" => self.wonder_room = false,

            // Other
            "gravity" => self.gravity = false,
            "mudsport" => self.mud_sport = false,
            "watersport" => self.water_sport = false,
            "iondeluge" => self.ion_deluge = false,
            "fairylock" => self.fairy_lock = false,

            _ => {}
        }
    }

    /// Check if any field condition is active
    pub fn has_any_condition(&self) -> bool {
        self.weather.is_some()
            || self.terrain.is_some()
            || self.trick_room
            || self.magic_room
            || self.wonder_room
            || self.gravity
            || self.mud_sport
            || self.water_sport
            || self.ion_deluge
            || self.fairy_lock
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_field_is_empty() {
        let field = FieldState::new();
        assert!(field.weather.is_none());
        assert!(field.terrain.is_none());
        assert!(!field.trick_room);
        assert!(!field.has_any_condition());
    }

    #[test]
    fn test_apply_field_start_terrain() {
        let mut field = FieldState::new();
        field.apply_field_start("Electric Terrain");
        assert_eq!(field.terrain, Some(Terrain::Electric));

        field.apply_field_start("move: Grassy Terrain");
        assert_eq!(field.terrain, Some(Terrain::Grassy));
    }

    #[test]
    fn test_apply_field_start_rooms() {
        let mut field = FieldState::new();

        field.apply_field_start("Trick Room");
        assert!(field.trick_room);

        field.apply_field_start("Magic Room");
        assert!(field.magic_room);

        field.apply_field_start("Wonder Room");
        assert!(field.wonder_room);
    }

    #[test]
    fn test_apply_field_start_gravity() {
        let mut field = FieldState::new();
        field.apply_field_start("Gravity");
        assert!(field.gravity);
    }

    #[test]
    fn test_apply_field_end() {
        let mut field = FieldState::new();
        field.trick_room = true;
        field.terrain = Some(Terrain::Electric);
        field.gravity = true;

        field.apply_field_end("Trick Room");
        assert!(!field.trick_room);

        field.apply_field_end("Electric Terrain");
        assert!(field.terrain.is_none());

        field.apply_field_end("Gravity");
        assert!(!field.gravity);
    }

    #[test]
    fn test_clear() {
        let mut field = FieldState {
            weather: Some(Weather::Sun),
            terrain: Some(Terrain::Grassy),
            trick_room: true,
            magic_room: true,
            wonder_room: false,
            gravity: true,
            mud_sport: false,
            water_sport: false,
            ion_deluge: false,
            fairy_lock: false,
        };

        field.clear();
        assert!(!field.has_any_condition());
    }

    #[test]
    fn test_has_any_condition() {
        let mut field = FieldState::new();
        assert!(!field.has_any_condition());

        field.weather = Some(Weather::Rain);
        assert!(field.has_any_condition());

        field.weather = None;
        field.trick_room = true;
        assert!(field.has_any_condition());
    }
}
