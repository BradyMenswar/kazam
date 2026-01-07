use crate::RoomState;
use kazam_protocol::{
    BattleInfo, BattleRequest, ChallengeState, FormatSection, HpStatus, Pokemon, PokemonDetails,
    RoomType, SearchState, ServerMessage, Side, Stat, User,
};

#[allow(async_fn_in_trait)]
pub trait KazamHandler: Send {
    // ===================
    // Global Messages
    // ===================

    async fn on_challstr(&mut self, challstr: &str) {
        let _ = challstr;
    }

    async fn on_update_user(&mut self, user: &User, named: bool, avatar: &str) {
        let _ = (user, named, avatar);
    }

    async fn on_name_taken(&mut self, username: &str, message: &str) {
        let _ = (username, message);
    }

    /// Called when |popup|MESSAGE is received (|| denotes newline)
    async fn on_popup(&mut self, message: &str) {
        let _ = message;
    }

    /// Called when |pm|SENDER|RECEIVER|MESSAGE is received
    async fn on_pm(&mut self, sender: &User, receiver: &User, message: &str) {
        let _ = (sender, receiver, message);
    }

    /// Called when |usercount|USERCOUNT is received
    async fn on_usercount(&mut self, count: u32) {
        let _ = count;
    }

    /// Called when |formats|FORMATSLIST is received
    async fn on_formats(&mut self, sections: &[FormatSection]) {
        let _ = sections;
    }

    /// Called when |updatesearch|JSON is received
    async fn on_update_search(&mut self, state: &SearchState) {
        let _ = state;
    }

    /// Called when |updatechallenges|JSON is received
    async fn on_update_challenges(&mut self, state: &ChallengeState) {
        let _ = state;
    }

    /// Called once when login succeeds (named becomes true for the first time)
    async fn on_logged_in(&mut self, user: &User) {
        let _ = user;
    }

    // ===================
    // Room Messages
    // ===================

    /// Called when |init|ROOMTYPE is received
    async fn on_init(&mut self, room_id: &str, room_type: &RoomType) {
        let _ = (room_id, room_type);
    }

    /// Called when |title|TITLE is received
    async fn on_title(&mut self, room_id: &str, title: &str) {
        let _ = (room_id, title);
    }

    /// Called when |users|USERLIST is received
    async fn on_users(&mut self, room_id: &str, users: &[User]) {
        let _ = (room_id, users);
    }

    /// Called after room initialization is complete (init + title + users received)
    async fn on_room_joined(&mut self, room: &RoomState) {
        let _ = room;
    }

    async fn on_join(&mut self, room_id: Option<&str>, user: &User, quiet: bool) {
        let _ = (room_id, user, quiet);
    }

    async fn on_leave(&mut self, room_id: Option<&str>, user: &User, quiet: bool) {
        let _ = (room_id, user, quiet);
    }

    async fn on_chat(
        &mut self,
        room_id: Option<&str>,
        user: &User,
        message: &str,
        timestamp: Option<i64>,
    ) {
        let _ = (room_id, user, message, timestamp);
    }

    /// Called when |:|TIMESTAMP is received (server's current time)
    async fn on_timestamp(&mut self, timestamp: i64) {
        let _ = timestamp;
    }

    /// Called when |battle|ROOMID|USER1|USER2 is received
    async fn on_battle(&mut self, room_id: &str, user1: &User, user2: &User) {
        let _ = (room_id, user1, user2);
    }

    /// Called when |notify|TITLE|MESSAGE or |notify|TITLE|MESSAGE|HIGHLIGHTTOKEN is received
    async fn on_notify(
        &mut self,
        title: &str,
        message: Option<&str>,
        highlight_token: Option<&str>,
    ) {
        let _ = (title, message, highlight_token);
    }

    /// Called when |name|USER|OLDID is received (user changed name)
    async fn on_name(&mut self, room_id: Option<&str>, user: &User, old_id: &str, quiet: bool) {
        let _ = (room_id, user, old_id, quiet);
    }

    /// Called when |html|HTML is received
    async fn on_html(&mut self, room_id: Option<&str>, html: &str) {
        let _ = (room_id, html);
    }

    /// Called when |uhtml|NAME|HTML is received (named, updatable HTML)
    async fn on_uhtml(&mut self, room_id: Option<&str>, name: &str, html: &str) {
        let _ = (room_id, name, html);
    }

    /// Called when |uhtmlchange|NAME|HTML is received (update existing uhtml)
    async fn on_uhtml_change(&mut self, room_id: Option<&str>, name: &str, html: &str) {
        let _ = (room_id, name, html);
    }

    async fn on_raw(&mut self, room_id: Option<&str>, content: &str) {
        let _ = (room_id, content);
    }

    // ===================
    // Battle Events - High Level
    // ===================

    /// Called when battle initialization is complete (all player/teamsize/gametype/rules received + |start|)
    async fn on_battle_started(&mut self, room_id: &str, battle: &BattleInfo) {
        let _ = (room_id, battle);
    }

    /// Called when a battle request is received (player needs to make a decision)
    async fn on_request(&mut self, room_id: &str, request: &BattleRequest) {
        let _ = (room_id, request);
    }

    /// Called when |turn|NUMBER is received
    async fn on_turn(&mut self, room_id: &str, turn: u32) {
        let _ = (room_id, turn);
    }

    /// Called when |win|USER is received
    async fn on_win(&mut self, room_id: &str, winner: &str) {
        let _ = (room_id, winner);
    }

    /// Called when |tie| is received
    async fn on_tie(&mut self, room_id: &str) {
        let _ = room_id;
    }

    // ===================
    // Battle Events - Major Actions
    // ===================

    /// Called when |switch| or |drag| is received
    async fn on_switch(
        &mut self,
        room_id: &str,
        pokemon: &Pokemon,
        details: &PokemonDetails,
        hp_status: Option<&HpStatus>,
        is_drag: bool,
    ) {
        let _ = (room_id, pokemon, details, hp_status, is_drag);
    }

    /// Called when |move| is received
    async fn on_move_used(
        &mut self,
        room_id: &str,
        pokemon: &Pokemon,
        move_name: &str,
        target: Option<&Pokemon>,
    ) {
        let _ = (room_id, pokemon, move_name, target);
    }

    /// Called when |faint| is received
    async fn on_faint(&mut self, room_id: &str, pokemon: &Pokemon) {
        let _ = (room_id, pokemon);
    }

    /// Called when |cant| is received (pokemon couldn't move)
    async fn on_cant(
        &mut self,
        room_id: &str,
        pokemon: &Pokemon,
        reason: &str,
        move_name: Option<&str>,
    ) {
        let _ = (room_id, pokemon, reason, move_name);
    }

    // ===================
    // Battle Events - Damage/Healing
    // ===================

    /// Called when |-damage| is received
    async fn on_damage(&mut self, room_id: &str, pokemon: &Pokemon, hp_status: Option<&HpStatus>) {
        let _ = (room_id, pokemon, hp_status);
    }

    /// Called when |-heal| is received
    async fn on_heal(&mut self, room_id: &str, pokemon: &Pokemon, hp_status: Option<&HpStatus>) {
        let _ = (room_id, pokemon, hp_status);
    }

    // ===================
    // Battle Events - Status
    // ===================

    /// Called when |-status| is received
    async fn on_status(&mut self, room_id: &str, pokemon: &Pokemon, status: &str) {
        let _ = (room_id, pokemon, status);
    }

    /// Called when |-curestatus| is received
    async fn on_cure_status(&mut self, room_id: &str, pokemon: &Pokemon, status: &str) {
        let _ = (room_id, pokemon, status);
    }

    // ===================
    // Battle Events - Stat Changes
    // ===================

    /// Called when |-boost| is received
    async fn on_boost(&mut self, room_id: &str, pokemon: &Pokemon, stat: Stat, amount: i8) {
        let _ = (room_id, pokemon, stat, amount);
    }

    /// Called when |-unboost| is received
    async fn on_unboost(&mut self, room_id: &str, pokemon: &Pokemon, stat: Stat, amount: i8) {
        let _ = (room_id, pokemon, stat, amount);
    }

    // ===================
    // Battle Events - Field Conditions
    // ===================

    /// Called when |-weather| is received
    async fn on_weather(&mut self, room_id: &str, weather: &str, upkeep: bool) {
        let _ = (room_id, weather, upkeep);
    }

    /// Called when |-fieldstart| is received
    async fn on_field_start(&mut self, room_id: &str, condition: &str) {
        let _ = (room_id, condition);
    }

    /// Called when |-fieldend| is received
    async fn on_field_end(&mut self, room_id: &str, condition: &str) {
        let _ = (room_id, condition);
    }

    /// Called when |-sidestart| is received
    async fn on_side_start(&mut self, room_id: &str, side: &Side, condition: &str) {
        let _ = (room_id, side, condition);
    }

    /// Called when |-sideend| is received
    async fn on_side_end(&mut self, room_id: &str, side: &Side, condition: &str) {
        let _ = (room_id, side, condition);
    }

    // ===================
    // Battle Events - Effectiveness
    // ===================

    /// Called when |-crit| is received
    async fn on_crit(&mut self, room_id: &str, pokemon: &Pokemon) {
        let _ = (room_id, pokemon);
    }

    /// Called when |-supereffective| is received
    async fn on_super_effective(&mut self, room_id: &str, pokemon: &Pokemon) {
        let _ = (room_id, pokemon);
    }

    /// Called when |-resisted| is received
    async fn on_resisted(&mut self, room_id: &str, pokemon: &Pokemon) {
        let _ = (room_id, pokemon);
    }

    /// Called when |-immune| is received
    async fn on_immune(&mut self, room_id: &str, pokemon: &Pokemon) {
        let _ = (room_id, pokemon);
    }

    /// Called when |-miss| is received
    async fn on_miss(&mut self, room_id: &str, source: &Pokemon, target: Option<&Pokemon>) {
        let _ = (room_id, source, target);
    }

    /// Called when |-fail| is received
    async fn on_fail(&mut self, room_id: &str, pokemon: &Pokemon, action: Option<&str>) {
        let _ = (room_id, pokemon, action);
    }

    // ===================
    // Battle Events - Items/Abilities
    // ===================

    /// Called when |-item| is received (item revealed or gained)
    async fn on_item(&mut self, room_id: &str, pokemon: &Pokemon, item: &str, from: Option<&str>) {
        let _ = (room_id, pokemon, item, from);
    }

    /// Called when |-enditem| is received (item consumed or removed)
    async fn on_end_item(
        &mut self,
        room_id: &str,
        pokemon: &Pokemon,
        item: &str,
        from: Option<&str>,
        eaten: bool,
    ) {
        let _ = (room_id, pokemon, item, from, eaten);
    }

    /// Called when |-ability| is received (ability revealed or changed)
    async fn on_ability(
        &mut self,
        room_id: &str,
        pokemon: &Pokemon,
        ability: &str,
        from: Option<&str>,
    ) {
        let _ = (room_id, pokemon, ability, from);
    }

    /// Called when |-endability| is received (ability suppressed)
    async fn on_end_ability(&mut self, room_id: &str, pokemon: &Pokemon) {
        let _ = (room_id, pokemon);
    }

    // ===================
    // Battle Events - Transformations
    // ===================

    /// Called when |-mega| is received
    async fn on_mega(&mut self, room_id: &str, pokemon: &Pokemon, megastone: &str) {
        let _ = (room_id, pokemon, megastone);
    }

    /// Called when |-primal| is received
    async fn on_primal(&mut self, room_id: &str, pokemon: &Pokemon) {
        let _ = (room_id, pokemon);
    }

    /// Called when |-zpower| is received
    async fn on_z_power(&mut self, room_id: &str, pokemon: &Pokemon) {
        let _ = (room_id, pokemon);
    }

    /// Called when |-burst| is received (Ultra Burst)
    async fn on_ultra_burst(&mut self, room_id: &str, pokemon: &Pokemon, species: &str, item: &str) {
        let _ = (room_id, pokemon, species, item);
    }

    /// Called when |-transform| is received
    async fn on_transform(&mut self, room_id: &str, pokemon: &Pokemon, into_species: &str) {
        let _ = (room_id, pokemon, into_species);
    }

    // ===================
    // Battle Events - Timer
    // ===================

    /// Called when |inactive| is received (timer warning)
    async fn on_inactive(&mut self, room_id: &str, message: &str) {
        let _ = (room_id, message);
    }

    /// Called when |inactiveoff| is received (timer turned off)
    async fn on_inactive_off(&mut self, room_id: &str, message: &str) {
        let _ = (room_id, message);
    }

    // ===================
    // Battle Events - Other
    // ===================

    /// Called when |-activate| is received (misc effect activation)
    async fn on_activate(&mut self, room_id: &str, pokemon: Option<&Pokemon>, effect: &str) {
        let _ = (room_id, pokemon, effect);
    }

    /// Called when |-hint| is received
    async fn on_hint(&mut self, room_id: &str, message: &str) {
        let _ = (room_id, message);
    }

    /// Called when |-message| is received
    async fn on_battle_message_text(&mut self, room_id: &str, message: &str) {
        let _ = (room_id, message);
    }

    /// Called for all battle-specific messages (catch-all for unhandled messages)
    /// This is called AFTER any specific handler above
    async fn on_battle_message(&mut self, room_id: Option<&str>, message: ServerMessage) {
        let _ = (room_id, message);
    }
}
