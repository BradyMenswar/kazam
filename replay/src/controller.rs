use std::time::Duration;

use kazam_battle::TrackedBattle;
use kazam_protocol::Player;

use crate::{ReplayError, ReplayLog};

/// Playback speed expressed as protocol messages per second.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReplaySpeed {
    messages_per_second: f64,
}

impl ReplaySpeed {
    pub fn new(messages_per_second: f64) -> Result<Self, ReplayError> {
        if !messages_per_second.is_finite() || messages_per_second < 0.0 {
            return Err(ReplayError::InvalidSpeed);
        }

        Ok(Self {
            messages_per_second,
        })
    }

    pub fn messages_per_second(self) -> f64 {
        self.messages_per_second
    }
}

impl Default for ReplaySpeed {
    fn default() -> Self {
        Self {
            messages_per_second: 8.0,
        }
    }
}

/// Stateful replay navigator built on top of `kazam-battle` snapshots.
#[derive(Debug, Clone)]
pub struct ReplayController {
    replay: ReplayLog,
    battle: TrackedBattle,
    applied_messages: usize,
    paused: bool,
    speed: ReplaySpeed,
    message_credit: f64,
}

impl ReplayController {
    pub fn new(replay: ReplayLog) -> Self {
        let battle = TrackedBattle::from_snapshot(&replay.turn_snapshots()[0].snapshot);

        Self {
            replay,
            battle,
            applied_messages: 0,
            paused: true,
            speed: ReplaySpeed::default(),
            message_credit: 0.0,
        }
    }

    pub fn replay(&self) -> &ReplayLog {
        &self.replay
    }

    pub fn battle(&self) -> &TrackedBattle {
        &self.battle
    }

    /// Number of replay messages already applied to the current battle state.
    pub fn applied_messages(&self) -> usize {
        self.applied_messages
    }

    pub fn total_messages(&self) -> usize {
        self.replay.len()
    }

    pub fn current_turn(&self) -> u32 {
        self.battle.turn
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn speed(&self) -> ReplaySpeed {
        self.speed
    }

    pub fn play(&mut self) {
        self.paused = false;
    }

    pub fn pause(&mut self) {
        self.paused = true;
        self.message_credit = 0.0;
    }

    pub fn set_speed(&mut self, speed: ReplaySpeed) {
        self.speed = speed;
    }

    pub fn set_viewpoint(&mut self, player: Player) {
        self.battle.set_viewpoint(player);
    }

    pub fn clear_viewpoint(&mut self) {
        self.battle.clear_viewpoint();
    }

    pub fn step_message(&mut self) -> bool {
        if self.applied_messages >= self.replay.len() {
            return false;
        }

        let event = &self.replay.events()[self.applied_messages];
        self.battle.apply_message(&event.message);
        self.applied_messages += 1;
        true
    }

    /// Advance playback according to the configured speed.
    ///
    /// Returns the number of messages applied during this tick.
    pub fn advance_by(&mut self, elapsed: Duration) -> usize {
        if self.paused || self.applied_messages >= self.replay.len() {
            return 0;
        }

        self.message_credit += elapsed.as_secs_f64() * self.speed.messages_per_second();

        let mut advanced = 0;
        while self.message_credit >= 1.0 && self.step_message() {
            self.message_credit -= 1.0;
            advanced += 1;
        }

        advanced
    }

    pub fn seek_message(&mut self, message_index: usize) -> Result<(), ReplayError> {
        if message_index > self.replay.len() {
            return Err(ReplayError::InvalidMessageIndex {
                index: message_index,
                len: self.replay.len(),
            });
        }

        let viewpoint = self.battle.viewpoint();
        let anchor = self.replay.anchor_for_message(message_index)?;
        self.battle = TrackedBattle::from_snapshot(&anchor.snapshot);

        if let Some(player) = viewpoint {
            self.battle.set_viewpoint(player);
        }

        for event in &self.replay.events()[anchor.message_index..message_index] {
            self.battle.apply_message(&event.message);
        }

        self.applied_messages = message_index;
        self.message_credit = 0.0;
        Ok(())
    }

    pub fn first_turn(&mut self) -> Result<(), ReplayError> {
        self.go_to_turn(1)
    }

    pub fn go_to_turn(&mut self, turn: u32) -> Result<(), ReplayError> {
        let snapshot = self.replay.snapshot_for_turn(turn)?;
        self.seek_message(snapshot.message_index)
    }

    pub fn previous_turn(&mut self) -> bool {
        let current_idx = self.current_turn_snapshot_index();
        if current_idx == 0 {
            return false;
        }

        let target_idx = if self.applied_messages
            == self.replay.turn_snapshots()[current_idx].message_index
        {
            current_idx - 1
        } else {
            current_idx
        };

        let target = self.replay.turn_snapshots()[target_idx].message_index;
        self.seek_message(target).is_ok()
    }

    pub fn next_turn(&mut self) -> bool {
        let current_idx = self.current_turn_snapshot_index();
        let target_idx = current_idx + 1;

        if target_idx >= self.replay.turn_snapshots().len() {
            return false;
        }

        let target = self.replay.turn_snapshots()[target_idx].message_index;
        self.seek_message(target).is_ok()
    }

    pub fn skip_turn(&mut self) -> bool {
        self.next_turn()
    }

    pub fn skip_to_end(&mut self) {
        let viewpoint = self.battle.viewpoint();
        self.battle = TrackedBattle::from_snapshot(self.replay.final_snapshot());
        if let Some(player) = viewpoint {
            self.battle.set_viewpoint(player);
        }
        self.applied_messages = self.replay.len();
        self.message_credit = 0.0;
    }

    fn current_turn_snapshot_index(&self) -> usize {
        self.replay
            .turn_snapshots()
            .iter()
            .rposition(|snapshot| snapshot.message_index <= self.applied_messages)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use kazam_battle::SideCondition;
    use kazam_protocol::Player;

    use super::*;

    fn sample_log() -> ReplayLog {
        ReplayLog::from_str(
            "|player|p1|Alice|1\n|player|p2|Bob|2\n|gametype|singles\n|start\n|switch|p1a: Lead|Skarmory, F|334/334\n|switch|p2a: Lead|Snorlax, M|497/497\n|turn|1\n|move|p1a: Lead|Spikes|p2a: Lead\n|-sidestart|p2: Bob|Spikes\n|turn|2\n|move|p2a: Lead|Body Slam|p1a: Lead\n|turn|3\n|win|Alice",
        )
        .unwrap()
    }

    #[test]
    fn test_turn_navigation() {
        let replay = sample_log();
        let mut controller = ReplayController::new(replay);

        controller.set_viewpoint(Player::P1);

        controller.first_turn().unwrap();
        assert_eq!(controller.current_turn(), 1);
        assert_eq!(controller.applied_messages(), 7);
        assert_eq!(controller.battle().viewpoint(), Some(Player::P1));

        assert!(controller.next_turn());
        assert_eq!(controller.current_turn(), 2);
        assert_eq!(controller.applied_messages(), 10);

        assert!(controller.previous_turn());
        assert_eq!(controller.current_turn(), 1);

        controller.skip_to_end();
        assert_eq!(controller.applied_messages(), controller.total_messages());
        assert_eq!(controller.battle().winner.as_deref(), Some("Alice"));
        assert_eq!(controller.battle().viewpoint(), Some(Player::P1));
    }

    #[test]
    fn test_seek_message_rebuilds_from_snapshots() {
        let replay = sample_log();
        let mut controller = ReplayController::new(replay);

        controller.seek_message(9).unwrap();

        assert_eq!(
            controller
                .battle()
                .get_side(Player::P2)
                .unwrap()
                .condition_layers(SideCondition::Spikes),
            1
        );
        assert_eq!(controller.current_turn(), 1);

        controller.seek_message(10).unwrap();
        assert_eq!(controller.current_turn(), 2);
    }

    #[test]
    fn test_advance_by_speed() {
        let replay = sample_log();
        let mut controller = ReplayController::new(replay);
        controller.set_speed(ReplaySpeed::new(4.0).unwrap());
        controller.play();

        assert_eq!(controller.advance_by(Duration::from_millis(250)), 1);
        assert_eq!(controller.applied_messages(), 1);

        assert_eq!(controller.advance_by(Duration::from_millis(500)), 2);
        assert_eq!(controller.applied_messages(), 3);

        controller.pause();
        assert_eq!(controller.advance_by(Duration::from_secs(1)), 0);
        assert_eq!(controller.applied_messages(), 3);
    }
}
