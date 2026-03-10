//! Snapshot helpers for reduced battle state

use kazam_protocol::ServerMessage;

use super::battle::{BattleKnowledge, TrackedBattle};

/// A captured copy of reduced battle state that can be restored later.
#[derive(Debug, Clone)]
pub struct BattleSnapshot {
    battle: TrackedBattle,
}

impl BattleSnapshot {
    /// Borrow the captured battle state.
    pub fn battle(&self) -> &TrackedBattle {
        &self.battle
    }

    /// Consume the snapshot and return the captured battle state.
    pub fn into_battle(self) -> TrackedBattle {
        self.battle
    }

    /// Turn number represented by this snapshot.
    pub fn turn(&self) -> u32 {
        self.battle.turn
    }

    /// Knowledge mode represented by this snapshot.
    pub fn knowledge(&self) -> BattleKnowledge {
        self.battle.knowledge()
    }
}

impl From<TrackedBattle> for BattleSnapshot {
    fn from(battle: TrackedBattle) -> Self {
        Self { battle }
    }
}

/// A snapshot captured at a turn boundary.
#[derive(Debug, Clone)]
pub struct TurnSnapshot {
    /// Number of messages applied before this snapshot was captured.
    pub message_index: usize,
    /// Battle turn number represented by the snapshot.
    pub turn: u32,
    /// Captured battle state.
    pub snapshot: BattleSnapshot,
}

impl TrackedBattle {
    /// Capture the current battle state as a reusable snapshot.
    pub fn snapshot(&self) -> BattleSnapshot {
        BattleSnapshot {
            battle: self.clone(),
        }
    }

    /// Restore this battle state from a previously captured snapshot.
    pub fn restore(&mut self, snapshot: &BattleSnapshot) {
        *self = snapshot.battle.clone();
    }

    /// Clone a new battle state from a snapshot.
    pub fn from_snapshot(snapshot: &BattleSnapshot) -> Self {
        snapshot.battle.clone()
    }

    /// Build snapshots at the start state and after each `|turn|` message.
    ///
    /// The returned snapshots are keyed by applied message count so consumers can
    /// jump near a turn boundary and replay the remaining in-turn messages.
    pub fn turn_snapshots<'a, I>(&self, messages: I) -> Vec<TurnSnapshot>
    where
        I: IntoIterator<Item = &'a ServerMessage>,
    {
        let mut battle = self.clone();
        let mut snapshots = vec![TurnSnapshot {
            message_index: 0,
            turn: battle.turn,
            snapshot: battle.snapshot(),
        }];

        for (index, message) in messages.into_iter().enumerate() {
            battle.apply_message(message);

            if let ServerMessage::Turn(turn) = message {
                snapshots.push(TurnSnapshot {
                    message_index: index + 1,
                    turn: *turn,
                    snapshot: battle.snapshot(),
                });
            }
        }

        snapshots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kazam_protocol::{Player, parse_server_message};

    #[test]
    fn test_snapshot_round_trip() {
        let mut battle = TrackedBattle::omniscient();
        battle.set_viewpoint(Player::P1);
        battle.apply_message(&parse_server_message("|player|p1|Alice|1").unwrap());
        battle.apply_message(&parse_server_message("|turn|3").unwrap());

        let snapshot = battle.snapshot();

        battle.apply_message(&parse_server_message("|win|Alice").unwrap());
        assert!(battle.ended);

        battle.restore(&snapshot);
        assert!(!battle.ended);
        assert_eq!(battle.turn, 3);
        assert_eq!(battle.viewpoint(), Some(Player::P1));
        assert_eq!(snapshot.turn(), 3);
        assert_eq!(snapshot.knowledge(), BattleKnowledge::Omniscient);
    }

    #[test]
    fn test_turn_snapshots_capture_turn_boundaries() {
        let messages = [
            parse_server_message("|player|p1|Alice|1").unwrap(),
            parse_server_message("|player|p2|Bob|2").unwrap(),
            parse_server_message("|turn|1").unwrap(),
            parse_server_message("|switch|p1a: Lead|Pikachu, M|100/100").unwrap(),
            parse_server_message("|turn|2").unwrap(),
        ];

        let battle = TrackedBattle::new();
        let snapshots = battle.turn_snapshots(messages.iter());

        assert_eq!(snapshots.len(), 3);
        assert_eq!(snapshots[0].message_index, 0);
        assert_eq!(snapshots[0].turn, 0);

        assert_eq!(snapshots[1].message_index, 3);
        assert_eq!(snapshots[1].turn, 1);
        assert_eq!(snapshots[1].snapshot.battle().turn, 1);

        assert_eq!(snapshots[2].message_index, 5);
        assert_eq!(snapshots[2].turn, 2);
        assert_eq!(snapshots[2].snapshot.battle().turn, 2);
        assert_eq!(snapshots[2].snapshot.battle().get_side(Player::P1).unwrap().username, "Alice");
    }
}
