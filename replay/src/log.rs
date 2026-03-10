use std::fs;
use std::path::Path;

use kazam_battle::{BattleSnapshot, TrackedBattle, TurnSnapshot};
use kazam_protocol::{ServerMessage, parse_server_message};

use crate::ReplayError;

/// One parsed replay message together with its original source line.
#[derive(Debug, Clone)]
pub struct ReplayEvent {
    pub index: usize,
    pub line_number: usize,
    pub raw_line: String,
    pub message: ServerMessage,
}

/// A parsed replay transcript with turn indexing and battle snapshots.
#[derive(Debug, Clone)]
pub struct ReplayLog {
    events: Vec<ReplayEvent>,
    turn_snapshots: Vec<TurnSnapshot>,
    final_snapshot: BattleSnapshot,
}

impl ReplayLog {
    /// Parse a replay transcript from a string.
    pub fn from_str(log: &str) -> Result<Self, ReplayError> {
        let events = log
            .lines()
            .enumerate()
            .map(|(line_idx, raw_line)| {
                parse_server_message(raw_line)
                    .map(|message| ReplayEvent {
                        index: line_idx,
                        line_number: line_idx + 1,
                        raw_line: raw_line.to_string(),
                        message,
                    })
                    .map_err(|source| ReplayError::ParseLine {
                        line_number: line_idx + 1,
                        raw_line: raw_line.to_string(),
                        source,
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Self::from_events(events)
    }

    /// Read and parse a replay transcript from disk.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ReplayError> {
        let path = path.as_ref();
        let log = fs::read_to_string(path).map_err(|source| ReplayError::ReadLog {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_str(&log)
    }

    /// Build a replay log from already-parsed events.
    pub fn from_events(events: Vec<ReplayEvent>) -> Result<Self, ReplayError> {
        let messages = events.iter().map(|event| &event.message);
        let root = TrackedBattle::omniscient();
        let turn_snapshots = root.turn_snapshots(messages);

        let mut final_battle = TrackedBattle::omniscient();
        final_battle.apply_messages(events.iter().map(|event| &event.message));

        Ok(Self {
            events,
            turn_snapshots,
            final_snapshot: final_battle.snapshot(),
        })
    }

    pub fn events(&self) -> &[ReplayEvent] {
        &self.events
    }

    pub fn event(&self, index: usize) -> Option<&ReplayEvent> {
        self.events.get(index)
    }

    /// Number of parsed replay messages.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Maximum turn number present in the replay.
    pub fn max_turn(&self) -> u32 {
        self.turn_snapshots.last().map(|snapshot| snapshot.turn).unwrap_or(0)
    }

    /// Turn boundary snapshots captured from the replay transcript.
    pub fn turn_snapshots(&self) -> &[TurnSnapshot] {
        &self.turn_snapshots
    }

    /// Final reduced battle state after all replay messages are applied.
    pub fn final_snapshot(&self) -> &BattleSnapshot {
        &self.final_snapshot
    }

    pub(crate) fn anchor_for_message(&self, message_index: usize) -> Result<&TurnSnapshot, ReplayError> {
        if message_index > self.events.len() {
            return Err(ReplayError::InvalidMessageIndex {
                index: message_index,
                len: self.events.len(),
            });
        }

        Ok(self
            .turn_snapshots
            .iter()
            .rfind(|snapshot| snapshot.message_index <= message_index)
            .unwrap_or(&self.turn_snapshots[0]))
    }

    pub(crate) fn snapshot_for_turn(&self, turn: u32) -> Result<&TurnSnapshot, ReplayError> {
        self.turn_snapshots
            .iter()
            .find(|snapshot| snapshot.turn == turn)
            .ok_or(ReplayError::TurnNotFound { turn })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kazam_battle::{SideCondition, Weather};
    use kazam_protocol::Player;

    fn sample_log() -> &'static str {
        "|player|p1|Alice|1\n|player|p2|Bob|2\n|gametype|singles\n|start\n|switch|p1a: Lead|Skarmory, F|334/334\n|switch|p2a: Lead|Snorlax, M|497/497\n|turn|1\n|move|p1a: Lead|Spikes|p2a: Lead\n|-sidestart|p2: Bob|Spikes\n|turn|2\n|-weather|Sandstorm|[from] ability: Sand Stream|[of] p1a: Lead\n|win|Alice"
    }

    #[test]
    fn test_parse_replay_log() {
        let replay = ReplayLog::from_str(sample_log()).unwrap();

        assert_eq!(replay.len(), 12);
        assert_eq!(replay.max_turn(), 2);
        assert_eq!(replay.turn_snapshots().len(), 3);
        assert_eq!(replay.turn_snapshots()[1].turn, 1);
        assert_eq!(replay.turn_snapshots()[2].message_index, 10);

        let final_battle = replay.final_snapshot().battle();
        assert_eq!(final_battle.winner.as_deref(), Some("Alice"));
        assert_eq!(final_battle.field.weather, Some(Weather::Sand));
        assert_eq!(
            final_battle
                .get_side(Player::P2)
                .unwrap()
                .condition_layers(SideCondition::Spikes),
            1
        );
    }
}
