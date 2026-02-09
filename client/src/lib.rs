use std::sync::atomic::Ordering;
use std::sync::Arc;

use anyhow::Result;
use kazam_protocol::{ClientMessage, ServerFrame};
use tokio::sync::mpsc;

mod connection;
mod handle;
mod handler;
mod room;

use connection::{Connection, ReconnectPolicy};
use handle::ClientState;

pub use handle::KazamHandle;
pub use handler::KazamHandler;
pub use kazam_protocol::{
    ActivePokemon, BattleInfo, BattleRequest, ChallengeInfo, ChallengeState, Format, FormatSection,
    GameType, HpStatus, MaxMoveSlot, MaxMoves, MoveSlot, Player, PlayerInfo, Pokemon,
    PokemonDetails, PokemonStats, PreviewPokemon, RoomType, SearchState, ServerMessage, Side,
    SideInfo, SidePokemon, Stat, User, ZMoveInfo,
};
pub use room::RoomState;

pub const SHOWDOWN_URL: &str = "wss://sim3.psim.us/showdown/websocket";

pub struct KazamClient {
    connection: Connection,
    state: Arc<ClientState>,
    cmd_rx: mpsc::UnboundedReceiver<ClientMessage>,
    cmd_tx: mpsc::UnboundedSender<ClientMessage>,
}

impl KazamClient {
    pub async fn connect(url: &str) -> Result<Self> {
        let connection = Connection::connect(url.to_string(), ReconnectPolicy::default()).await?;
        let state = Arc::new(ClientState::new());
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

        Ok(Self {
            connection,
            state,
            cmd_rx,
            cmd_tx,
        })
    }

    pub fn handle(&self) -> KazamHandle {
        KazamHandle::new(self.cmd_tx.clone(), self.state.clone())
    }

    pub async fn run<H: KazamHandler>(&mut self, handler: &mut H) -> Result<()> {
        loop {
            tokio::select! {
                frame = self.connection.recv() => {
                    self.dispatch_frame(frame?, handler).await?;
                }

                cmd = self.cmd_rx.recv() => {
                    if let Some(cmd) = cmd {
                        self.handle_command(cmd).await?;
                    }
                }
            }
        }
    }

    async fn handle_command(&mut self, msg: ClientMessage) -> Result<()> {
        self.connection.send(msg.to_wire_format()).await
    }

    async fn dispatch_frame<H: KazamHandler>(
        &mut self,
        frame: ServerFrame,
        handler: &mut H,
    ) -> Result<()> {
        let room_id = frame.room_id.clone();

        for message in frame.messages {
            match message {
                ServerMessage::Challstr(challstr) => {
                    handler.on_challstr(&challstr).await;
                }

                ServerMessage::UpdateUser {
                    user,
                    named,
                    avatar,
                } => {
                    let was_logged_in = self.state.logged_in.load(Ordering::Relaxed);
                    if named {
                        self.state.logged_in.store(true, Ordering::Relaxed);
                    }
                    handler.on_update_user(&user, named, &avatar).await;
                    if named && !was_logged_in {
                        handler.on_logged_in(&user).await;
                    }
                }

                ServerMessage::NameTaken { username, message } => {
                    handler.on_name_taken(&username, &message).await;
                }

                ServerMessage::Popup(message) => {
                    handler.on_popup(&message).await;
                }

                ServerMessage::Pm {
                    sender,
                    receiver,
                    message,
                } => {
                    handler.on_pm(&sender, &receiver, &message).await;
                }

                ServerMessage::Usercount(count) => {
                    handler.on_usercount(count).await;
                }

                ServerMessage::Formats(sections) => {
                    handler.on_formats(&sections).await;
                }

                ServerMessage::UpdateSearch(state) => {
                    handler.on_update_search(&state).await;
                }

                ServerMessage::UpdateChallenges(state) => {
                    handler.on_update_challenges(&state).await;
                }

                ServerMessage::Init(room_type) => {
                    if let Some(ref rid) = room_id {
                        let state = RoomState {
                            id: rid.clone(),
                            room_type: room_type.clone(),
                            title: None,
                            users: vec![],
                        };
                        if let Ok(mut rooms) = self.state.rooms.write() {
                            rooms.insert(rid.clone(), state);
                        }
                        handler.on_init(rid, &room_type).await;
                    }
                }

                ServerMessage::Title(title) => {
                    if let Some(ref rid) = room_id {
                        if let Ok(mut rooms) = self.state.rooms.write()
                            && let Some(room) = rooms.get_mut(rid) {
                                room.title = Some(title.clone());
                            }
                        handler.on_title(rid, &title).await;
                    }
                }

                ServerMessage::Users(users) => {
                    if let Some(ref rid) = room_id {
                        let room_snapshot = if let Ok(mut rooms) = self.state.rooms.write() {
                            if let Some(room) = rooms.get_mut(rid) {
                                room.users = users.clone();
                                Some(room.clone())
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        handler.on_users(rid, &users).await;

                        if let Some(room) = room_snapshot {
                            handler.on_room_joined(&room).await;
                        }
                    }
                }

                ServerMessage::Join { user, quiet } => {
                    if let Some(ref rid) = room_id
                        && let Ok(mut rooms) = self.state.rooms.write()
                            && let Some(room) = rooms.get_mut(rid)
                                && !room.users.iter().any(|u| u.username == user.username) {
                                    room.users.push(user.clone());
                                }
                    handler.on_join(room_id.as_deref(), &user, quiet).await;
                }

                ServerMessage::Leave { user, quiet } => {
                    if let Some(ref rid) = room_id
                        && let Ok(mut rooms) = self.state.rooms.write()
                            && let Some(room) = rooms.get_mut(rid) {
                                room.users.retain(|u| u.username != user.username);
                            }
                    handler.on_leave(room_id.as_deref(), &user, quiet).await;
                }

                ServerMessage::Chat {
                    user,
                    message,
                    timestamp,
                } => {
                    handler
                        .on_chat(room_id.as_deref(), &user, &message, timestamp)
                        .await;
                }

                ServerMessage::Timestamp(timestamp) => {
                    handler.on_timestamp(timestamp).await;
                }

                ServerMessage::Battle {
                    room_id: battle_room_id,
                    user1,
                    user2,
                } => {
                    handler.on_battle(&battle_room_id, &user1, &user2).await;
                }

                ServerMessage::Notify {
                    title,
                    message,
                    highlight_token,
                } => {
                    handler
                        .on_notify(&title, message.as_deref(), highlight_token.as_deref())
                        .await;
                }

                ServerMessage::Name {
                    user,
                    old_id,
                    quiet,
                } => {
                    if let Some(ref rid) = room_id
                        && let Ok(mut rooms) = self.state.rooms.write()
                            && let Some(room) = rooms.get_mut(rid) {
                                // Update user in room's user list
                                if let Some(existing) = room
                                    .users
                                    .iter_mut()
                                    .find(|u| u.username.to_lowercase() == old_id.to_lowercase())
                                {
                                    *existing = user.clone();
                                }
                            }
                    handler
                        .on_name(room_id.as_deref(), &user, &old_id, quiet)
                        .await;
                }

                ServerMessage::Html(html) => {
                    handler.on_html(room_id.as_deref(), &html).await;
                }

                ServerMessage::Uhtml { name, html } => {
                    handler.on_uhtml(room_id.as_deref(), &name, &html).await;
                }

                ServerMessage::UhtmlChange { name, html } => {
                    handler
                        .on_uhtml_change(room_id.as_deref(), &name, &html)
                        .await;
                }

                ServerMessage::Raw(content) => {
                    handler.on_raw(room_id.as_deref(), &content).await;
                }

                // ===================
                // Battle Initialization
                // ===================
                ServerMessage::BattlePlayer {
                    player,
                    username,
                    avatar,
                    rating,
                } => {
                    if let Some(ref rid) = room_id
                        && let Ok(mut battles) = self.state.battles.write() {
                            let battle = battles.entry(rid.clone()).or_insert_with(BattleInfo::new);
                            battle.players.push(PlayerInfo {
                                player,
                                username: username.clone(),
                                avatar: avatar.clone(),
                                rating,
                                team_size: 0,
                            });
                        }
                    handler
                        .on_battle_message(room_id.as_deref(), ServerMessage::BattlePlayer {
                            player,
                            username,
                            avatar,
                            rating,
                        })
                        .await;
                }

                ServerMessage::TeamSize { player, size } => {
                    if let Some(ref rid) = room_id
                        && let Ok(mut battles) = self.state.battles.write()
                            && let Some(battle) = battles.get_mut(rid)
                                && let Some(p) = battle.players.iter_mut().find(|p| p.player == player) {
                                    p.team_size = size;
                                }
                    handler
                        .on_battle_message(room_id.as_deref(), ServerMessage::TeamSize { player, size })
                        .await;
                }

                ServerMessage::GameType(game_type) => {
                    if let Some(ref rid) = room_id
                        && let Ok(mut battles) = self.state.battles.write()
                            && let Some(battle) = battles.get_mut(rid) {
                                battle.game_type = Some(game_type);
                            }
                    handler
                        .on_battle_message(room_id.as_deref(), ServerMessage::GameType(game_type))
                        .await;
                }

                ServerMessage::Gen(generation) => {
                    if let Some(ref rid) = room_id
                        && let Ok(mut battles) = self.state.battles.write()
                            && let Some(battle) = battles.get_mut(rid) {
                                battle.generation = generation;
                            }
                    handler
                        .on_battle_message(room_id.as_deref(), ServerMessage::Gen(generation))
                        .await;
                }

                ServerMessage::Tier(tier) => {
                    if let Some(ref rid) = room_id
                        && let Ok(mut battles) = self.state.battles.write()
                            && let Some(battle) = battles.get_mut(rid) {
                                battle.tier = tier.clone();
                            }
                    handler
                        .on_battle_message(room_id.as_deref(), ServerMessage::Tier(tier))
                        .await;
                }

                ServerMessage::Rated(message) => {
                    if let Some(ref rid) = room_id
                        && let Ok(mut battles) = self.state.battles.write()
                            && let Some(battle) = battles.get_mut(rid) {
                                battle.rated = true;
                                battle.rated_message = message.clone();
                            }
                    handler
                        .on_battle_message(room_id.as_deref(), ServerMessage::Rated(message))
                        .await;
                }

                ServerMessage::Rule(rule) => {
                    if let Some(ref rid) = room_id
                        && let Ok(mut battles) = self.state.battles.write()
                            && let Some(battle) = battles.get_mut(rid) {
                                battle.rules.push(rule.clone());
                            }
                    handler
                        .on_battle_message(room_id.as_deref(), ServerMessage::Rule(rule))
                        .await;
                }

                ServerMessage::Poke {
                    player,
                    details,
                    has_item,
                } => {
                    if let Some(ref rid) = room_id
                        && let Ok(mut battles) = self.state.battles.write()
                            && let Some(battle) = battles.get_mut(rid) {
                                battle.preview.push(PreviewPokemon {
                                    player,
                                    species: details.species.clone(),
                                    level: details.level,
                                    gender: details.gender,
                                    has_item,
                                });
                            }
                    handler
                        .on_battle_message(
                            room_id.as_deref(),
                            ServerMessage::Poke {
                                player,
                                details,
                                has_item,
                            },
                        )
                        .await;
                }

                ServerMessage::BattleStart => {
                    let battle_snapshot = if let Some(ref rid) = room_id {
                        if let Ok(mut battles) = self.state.battles.write() {
                            if let Some(battle) = battles.get_mut(rid) {
                                battle.started = true;
                                Some(battle.clone())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    if let (Some(rid), Some(battle)) = (&room_id, battle_snapshot) {
                        handler.on_battle_started(rid, &battle).await;
                    }
                    handler
                        .on_battle_message(room_id.as_deref(), ServerMessage::BattleStart)
                        .await;
                }

                // ===================
                // Battle Progress
                // ===================
                ServerMessage::Request(ref json) => {
                    if let Some(ref rid) = room_id
                        && let Some(request) = BattleRequest::parse(json) {
                            handler.on_request(rid, &request).await;
                        }
                    handler
                        .on_battle_message(room_id.as_deref(), ServerMessage::Request(json.clone()))
                        .await;
                }

                ServerMessage::Turn(turn) => {
                    if let Some(ref rid) = room_id {
                        if let Ok(mut battles) = self.state.battles.write()
                            && let Some(battle) = battles.get_mut(rid) {
                                battle.turn = turn;
                            }
                        handler.on_turn(rid, turn).await;
                    }
                    handler
                        .on_battle_message(room_id.as_deref(), ServerMessage::Turn(turn))
                        .await;
                }

                ServerMessage::Win(ref winner) => {
                    if let Some(ref rid) = room_id {
                        if let Ok(mut battles) = self.state.battles.write()
                            && let Some(battle) = battles.get_mut(rid) {
                                battle.winner = Some(winner.clone());
                            }
                        handler.on_win(rid, winner).await;
                    }
                    handler
                        .on_battle_message(room_id.as_deref(), ServerMessage::Win(winner.clone()))
                        .await;
                }

                ServerMessage::Tie => {
                    if let Some(ref rid) = room_id {
                        if let Ok(mut battles) = self.state.battles.write()
                            && let Some(battle) = battles.get_mut(rid) {
                                battle.tie = true;
                            }
                        handler.on_tie(rid).await;
                    }
                    handler
                        .on_battle_message(room_id.as_deref(), ServerMessage::Tie)
                        .await;
                }

                ServerMessage::Inactive(ref message) => {
                    if let Some(ref rid) = room_id {
                        handler.on_inactive(rid, message).await;
                    }
                    handler
                        .on_battle_message(room_id.as_deref(), ServerMessage::Inactive(message.clone()))
                        .await;
                }

                ServerMessage::InactiveOff(ref message) => {
                    if let Some(ref rid) = room_id {
                        handler.on_inactive_off(rid, message).await;
                    }
                    handler
                        .on_battle_message(room_id.as_deref(), ServerMessage::InactiveOff(message.clone()))
                        .await;
                }

                // ===================
                // Major Actions
                // ===================
                ServerMessage::Switch {
                    ref pokemon,
                    ref details,
                    ref hp_status,
                } => {
                    if let Some(ref rid) = room_id {
                        handler
                            .on_switch(rid, pokemon, details, hp_status.as_ref(), false)
                            .await;
                    }
                    handler
                        .on_battle_message(
                            room_id.as_deref(),
                            ServerMessage::Switch {
                                pokemon: pokemon.clone(),
                                details: details.clone(),
                                hp_status: hp_status.clone(),
                            },
                        )
                        .await;
                }

                ServerMessage::Drag {
                    ref pokemon,
                    ref details,
                    ref hp_status,
                } => {
                    if let Some(ref rid) = room_id {
                        handler
                            .on_switch(rid, pokemon, details, hp_status.as_ref(), true)
                            .await;
                    }
                    handler
                        .on_battle_message(
                            room_id.as_deref(),
                            ServerMessage::Drag {
                                pokemon: pokemon.clone(),
                                details: details.clone(),
                                hp_status: hp_status.clone(),
                            },
                        )
                        .await;
                }

                ServerMessage::Move {
                    ref pokemon,
                    ref move_name,
                    ref target,
                    ..
                } => {
                    if let Some(ref rid) = room_id {
                        handler
                            .on_move_used(rid, pokemon, move_name, target.as_ref())
                            .await;
                    }
                    handler
                        .on_battle_message(room_id.as_deref(), message)
                        .await;
                }

                ServerMessage::Faint(ref pokemon) => {
                    if let Some(ref rid) = room_id {
                        handler.on_faint(rid, pokemon).await;
                    }
                    handler
                        .on_battle_message(room_id.as_deref(), ServerMessage::Faint(pokemon.clone()))
                        .await;
                }

                ServerMessage::Cant {
                    ref pokemon,
                    ref reason,
                    ref move_name,
                } => {
                    if let Some(ref rid) = room_id {
                        handler
                            .on_cant(rid, pokemon, reason, move_name.as_deref())
                            .await;
                    }
                    handler
                        .on_battle_message(
                            room_id.as_deref(),
                            ServerMessage::Cant {
                                pokemon: pokemon.clone(),
                                reason: reason.clone(),
                                move_name: move_name.clone(),
                            },
                        )
                        .await;
                }

                // ===================
                // Minor Actions
                // ===================
                ServerMessage::Damage {
                    ref pokemon,
                    ref hp_status,
                } => {
                    if let Some(ref rid) = room_id {
                        handler.on_damage(rid, pokemon, hp_status.as_ref()).await;
                    }
                    handler
                        .on_battle_message(
                            room_id.as_deref(),
                            ServerMessage::Damage {
                                pokemon: pokemon.clone(),
                                hp_status: hp_status.clone(),
                            },
                        )
                        .await;
                }

                ServerMessage::Heal {
                    ref pokemon,
                    ref hp_status,
                } => {
                    if let Some(ref rid) = room_id {
                        handler.on_heal(rid, pokemon, hp_status.as_ref()).await;
                    }
                    handler
                        .on_battle_message(
                            room_id.as_deref(),
                            ServerMessage::Heal {
                                pokemon: pokemon.clone(),
                                hp_status: hp_status.clone(),
                            },
                        )
                        .await;
                }

                ServerMessage::Status {
                    ref pokemon,
                    ref status,
                } => {
                    if let Some(ref rid) = room_id {
                        handler.on_status(rid, pokemon, status).await;
                    }
                    handler
                        .on_battle_message(
                            room_id.as_deref(),
                            ServerMessage::Status {
                                pokemon: pokemon.clone(),
                                status: status.clone(),
                            },
                        )
                        .await;
                }

                ServerMessage::CureStatus {
                    ref pokemon,
                    ref status,
                } => {
                    if let Some(ref rid) = room_id {
                        handler.on_cure_status(rid, pokemon, status).await;
                    }
                    handler
                        .on_battle_message(
                            room_id.as_deref(),
                            ServerMessage::CureStatus {
                                pokemon: pokemon.clone(),
                                status: status.clone(),
                            },
                        )
                        .await;
                }

                ServerMessage::Boost {
                    ref pokemon,
                    stat,
                    amount,
                } => {
                    if let Some(ref rid) = room_id {
                        handler.on_boost(rid, pokemon, stat, amount).await;
                    }
                    handler
                        .on_battle_message(
                            room_id.as_deref(),
                            ServerMessage::Boost {
                                pokemon: pokemon.clone(),
                                stat,
                                amount,
                            },
                        )
                        .await;
                }

                ServerMessage::Unboost {
                    ref pokemon,
                    stat,
                    amount,
                } => {
                    if let Some(ref rid) = room_id {
                        handler.on_unboost(rid, pokemon, stat, amount).await;
                    }
                    handler
                        .on_battle_message(
                            room_id.as_deref(),
                            ServerMessage::Unboost {
                                pokemon: pokemon.clone(),
                                stat,
                                amount,
                            },
                        )
                        .await;
                }

                ServerMessage::Weather { ref weather, upkeep } => {
                    if let Some(ref rid) = room_id {
                        handler.on_weather(rid, weather, upkeep).await;
                    }
                    handler
                        .on_battle_message(
                            room_id.as_deref(),
                            ServerMessage::Weather {
                                weather: weather.clone(),
                                upkeep,
                            },
                        )
                        .await;
                }

                ServerMessage::FieldStart(ref condition) => {
                    if let Some(ref rid) = room_id {
                        handler.on_field_start(rid, condition).await;
                    }
                    handler
                        .on_battle_message(room_id.as_deref(), ServerMessage::FieldStart(condition.clone()))
                        .await;
                }

                ServerMessage::FieldEnd(ref condition) => {
                    if let Some(ref rid) = room_id {
                        handler.on_field_end(rid, condition).await;
                    }
                    handler
                        .on_battle_message(room_id.as_deref(), ServerMessage::FieldEnd(condition.clone()))
                        .await;
                }

                ServerMessage::SideStart {
                    ref side,
                    ref condition,
                } => {
                    if let Some(ref rid) = room_id {
                        handler.on_side_start(rid, side, condition).await;
                    }
                    handler
                        .on_battle_message(
                            room_id.as_deref(),
                            ServerMessage::SideStart {
                                side: side.clone(),
                                condition: condition.clone(),
                            },
                        )
                        .await;
                }

                ServerMessage::SideEnd {
                    ref side,
                    ref condition,
                } => {
                    if let Some(ref rid) = room_id {
                        handler.on_side_end(rid, side, condition).await;
                    }
                    handler
                        .on_battle_message(
                            room_id.as_deref(),
                            ServerMessage::SideEnd {
                                side: side.clone(),
                                condition: condition.clone(),
                            },
                        )
                        .await;
                }

                ServerMessage::Crit(ref pokemon) => {
                    if let Some(ref rid) = room_id {
                        handler.on_crit(rid, pokemon).await;
                    }
                    handler
                        .on_battle_message(room_id.as_deref(), ServerMessage::Crit(pokemon.clone()))
                        .await;
                }

                ServerMessage::SuperEffective(ref pokemon) => {
                    if let Some(ref rid) = room_id {
                        handler.on_super_effective(rid, pokemon).await;
                    }
                    handler
                        .on_battle_message(room_id.as_deref(), ServerMessage::SuperEffective(pokemon.clone()))
                        .await;
                }

                ServerMessage::Resisted(ref pokemon) => {
                    if let Some(ref rid) = room_id {
                        handler.on_resisted(rid, pokemon).await;
                    }
                    handler
                        .on_battle_message(room_id.as_deref(), ServerMessage::Resisted(pokemon.clone()))
                        .await;
                }

                ServerMessage::Immune(ref pokemon) => {
                    if let Some(ref rid) = room_id {
                        handler.on_immune(rid, pokemon).await;
                    }
                    handler
                        .on_battle_message(room_id.as_deref(), ServerMessage::Immune(pokemon.clone()))
                        .await;
                }

                ServerMessage::Miss {
                    ref source,
                    ref target,
                } => {
                    if let Some(ref rid) = room_id {
                        handler.on_miss(rid, source, target.as_ref()).await;
                    }
                    handler
                        .on_battle_message(
                            room_id.as_deref(),
                            ServerMessage::Miss {
                                source: source.clone(),
                                target: target.clone(),
                            },
                        )
                        .await;
                }

                ServerMessage::Fail {
                    ref pokemon,
                    ref action,
                } => {
                    if let Some(ref rid) = room_id {
                        handler.on_fail(rid, pokemon, action.as_deref()).await;
                    }
                    handler
                        .on_battle_message(
                            room_id.as_deref(),
                            ServerMessage::Fail {
                                pokemon: pokemon.clone(),
                                action: action.clone(),
                            },
                        )
                        .await;
                }

                ServerMessage::Item {
                    ref pokemon,
                    ref item,
                    ref from,
                } => {
                    if let Some(ref rid) = room_id {
                        handler.on_item(rid, pokemon, item, from.as_deref()).await;
                    }
                    handler
                        .on_battle_message(
                            room_id.as_deref(),
                            ServerMessage::Item {
                                pokemon: pokemon.clone(),
                                item: item.clone(),
                                from: from.clone(),
                            },
                        )
                        .await;
                }

                ServerMessage::EndItem {
                    ref pokemon,
                    ref item,
                    ref from,
                    eat,
                } => {
                    if let Some(ref rid) = room_id {
                        handler
                            .on_end_item(rid, pokemon, item, from.as_deref(), eat)
                            .await;
                    }
                    handler
                        .on_battle_message(
                            room_id.as_deref(),
                            ServerMessage::EndItem {
                                pokemon: pokemon.clone(),
                                item: item.clone(),
                                from: from.clone(),
                                eat,
                            },
                        )
                        .await;
                }

                ServerMessage::Ability {
                    ref pokemon,
                    ref ability,
                    ref from,
                } => {
                    if let Some(ref rid) = room_id {
                        handler
                            .on_ability(rid, pokemon, ability, from.as_deref())
                            .await;
                    }
                    handler
                        .on_battle_message(
                            room_id.as_deref(),
                            ServerMessage::Ability {
                                pokemon: pokemon.clone(),
                                ability: ability.clone(),
                                from: from.clone(),
                            },
                        )
                        .await;
                }

                ServerMessage::EndAbility(ref pokemon) => {
                    if let Some(ref rid) = room_id {
                        handler.on_end_ability(rid, pokemon).await;
                    }
                    handler
                        .on_battle_message(room_id.as_deref(), ServerMessage::EndAbility(pokemon.clone()))
                        .await;
                }

                ServerMessage::Mega {
                    ref pokemon,
                    ref megastone,
                } => {
                    if let Some(ref rid) = room_id {
                        handler.on_mega(rid, pokemon, megastone).await;
                    }
                    handler
                        .on_battle_message(
                            room_id.as_deref(),
                            ServerMessage::Mega {
                                pokemon: pokemon.clone(),
                                megastone: megastone.clone(),
                            },
                        )
                        .await;
                }

                ServerMessage::Primal(ref pokemon) => {
                    if let Some(ref rid) = room_id {
                        handler.on_primal(rid, pokemon).await;
                    }
                    handler
                        .on_battle_message(room_id.as_deref(), ServerMessage::Primal(pokemon.clone()))
                        .await;
                }

                ServerMessage::ZPower(ref pokemon) => {
                    if let Some(ref rid) = room_id {
                        handler.on_z_power(rid, pokemon).await;
                    }
                    handler
                        .on_battle_message(room_id.as_deref(), ServerMessage::ZPower(pokemon.clone()))
                        .await;
                }

                ServerMessage::Burst {
                    ref pokemon,
                    ref species,
                    ref item,
                } => {
                    if let Some(ref rid) = room_id {
                        handler.on_ultra_burst(rid, pokemon, species, item).await;
                    }
                    handler
                        .on_battle_message(
                            room_id.as_deref(),
                            ServerMessage::Burst {
                                pokemon: pokemon.clone(),
                                species: species.clone(),
                                item: item.clone(),
                            },
                        )
                        .await;
                }

                ServerMessage::Transform {
                    ref pokemon,
                    ref species,
                } => {
                    if let Some(ref rid) = room_id {
                        handler.on_transform(rid, pokemon, species).await;
                    }
                    handler
                        .on_battle_message(
                            room_id.as_deref(),
                            ServerMessage::Transform {
                                pokemon: pokemon.clone(),
                                species: species.clone(),
                            },
                        )
                        .await;
                }

                ServerMessage::Activate {
                    ref pokemon,
                    ref effect,
                } => {
                    if let Some(ref rid) = room_id {
                        handler.on_activate(rid, pokemon.as_ref(), effect).await;
                    }
                    handler
                        .on_battle_message(
                            room_id.as_deref(),
                            ServerMessage::Activate {
                                pokemon: pokemon.clone(),
                                effect: effect.clone(),
                            },
                        )
                        .await;
                }

                ServerMessage::Hint(ref msg) => {
                    if let Some(ref rid) = room_id {
                        handler.on_hint(rid, msg).await;
                    }
                    handler
                        .on_battle_message(room_id.as_deref(), ServerMessage::Hint(msg.clone()))
                        .await;
                }

                ServerMessage::Message(ref msg) => {
                    if let Some(ref rid) = room_id {
                        handler.on_battle_message_text(rid, msg).await;
                    }
                    handler
                        .on_battle_message(room_id.as_deref(), ServerMessage::Message(msg.clone()))
                        .await;
                }

                // All other battle messages just go to on_battle_message
                other => {
                    handler.on_battle_message(room_id.as_deref(), other).await;
                }
            }
        }
        Ok(())
    }
}
