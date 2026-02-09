pub mod battle;
pub mod battle_state;
pub mod request;
mod battle_init;
mod battle_major;
mod battle_minor;
mod battle_progress;
mod global;
mod room;

use anyhow::Result;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

pub use battle::{GameType, HpStatus, Player, Pokemon, PokemonDetails, Side, Stat};
pub use battle_state::{BattleInfo, PlayerInfo, PreviewPokemon};
pub use request::{
    ActivePokemon, BattleRequest, MaxMoveSlot, MaxMoves, MoveSlot, PokemonStats, SideInfo,
    SidePokemon, ZMoveInfo,
};

#[derive(Debug, Clone, PartialEq)]
pub struct User {
    /// The user's rank (space for no rank, @, %, +, etc.)
    pub rank: char,
    pub username: String,
    pub away: bool,
}

impl User {
    /// Parse a user string in the format "RANKUSERNAME" or "RANKUSERNAME@STATUS"
    pub fn parse(user_str: &str) -> Option<Self> {
        if user_str.is_empty() {
            return None;
        }

        let mut chars = user_str.chars();
        let rank = chars.next()?;
        let rest: String = chars.collect();

        let (username, away) = if let Some(at_pos) = rest.rfind('@') {
            let name = &rest[..at_pos];
            let status = &rest[at_pos + 1..];
            (name.to_string(), status.starts_with('!'))
        } else {
            (rest, false)
        };

        Some(Self {
            rank,
            username,
            away,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServerMessage {
    /// |challstr|CHALLSTR
    Challstr(String),

    /// |updateuser|USER|NAMED|AVATAR|SETTINGS
    UpdateUser {
        user: User,
        named: bool,
        avatar: String,
    },

    /// |nametaken|USERNAME|MESSAGE
    NameTaken { username: String, message: String },

    /// |popup|MESSAGE (|| denotes newline)
    Popup(String),

    /// |pm|SENDER|RECEIVER|MESSAGE
    Pm {
        sender: User,
        receiver: User,
        message: String,
    },

    /// |usercount|USERCOUNT
    Usercount(u32),

    /// |formats|FORMATSLIST
    Formats(Vec<FormatSection>),

    /// |updatesearch|JSON
    UpdateSearch(SearchState),

    /// |updatechallenges|JSON
    UpdateChallenges(ChallengeState),

    /// |init|ROOMTYPE
    Init(RoomType),

    /// |title|TITLE
    Title(String),

    /// |users|USERLIST
    Users(Vec<User>),

    /// |join|USER, |j|USER, or |J|USER
    Join { user: User, quiet: bool },

    /// |leave|USER, |l|USER, or |L|USER
    Leave { user: User, quiet: bool },

    /// |chat|USER|MESSAGE, |c|USER|MESSAGE, or |c:|TIMESTAMP|USER|MESSAGE
    Chat {
        user: User,
        message: String,
        timestamp: Option<i64>,
    },

    /// |:|TIMESTAMP - server's current time
    Timestamp(i64),

    /// |battle|ROOMID|USER1|USER2 or |b|ROOMID|USER1|USER2
    Battle {
        room_id: String,
        user1: User,
        user2: User,
    },

    /// |notify|TITLE|MESSAGE or |notify|TITLE|MESSAGE|HIGHLIGHTTOKEN
    Notify {
        title: String,
        message: Option<String>,
        highlight_token: Option<String>,
    },

    /// |name|USER|OLDID, |n|USER|OLDID, or |N|USER|OLDID
    Name {
        user: User,
        old_id: String,
        quiet: bool,
    },

    /// |html|HTML
    Html(String),

    /// |uhtml|NAME|HTML
    Uhtml { name: String, html: String },

    /// |uhtmlchange|NAME|HTML
    UhtmlChange { name: String, html: String },

    // ===================
    // Battle Initialization
    // ===================
    /// |player|PLAYER|USERNAME|AVATAR|RATING
    BattlePlayer {
        player: Player,
        username: String,
        avatar: String,
        rating: Option<u32>,
    },

    /// |teamsize|PLAYER|NUMBER
    TeamSize { player: Player, size: u8 },

    /// |gametype|GAMETYPE
    GameType(GameType),

    /// |gen|GENNUM
    Gen(u8),

    /// |tier|FORMATNAME
    Tier(String),

    /// |rated| or |rated|MESSAGE
    Rated(Option<String>),

    /// |rule|RULE: DESCRIPTION
    Rule(String),

    /// |clearpoke - marks start of team preview
    ClearPoke,

    /// |poke|PLAYER|DETAILS|ITEM
    Poke {
        player: Player,
        details: PokemonDetails,
        has_item: bool,
    },

    /// |teampreview or |teampreview|NUMBER
    TeamPreview(Option<u8>),

    /// |start - indicates battle has started
    BattleStart,

    // ===================
    // Battle Progress
    // ===================
    /// |request|JSON
    Request(Value),

    /// |inactive|MESSAGE
    Inactive(String),

    /// |inactiveoff|MESSAGE
    InactiveOff(String),

    /// |upkeep
    Upkeep,

    /// |turn|NUMBER
    Turn(u32),

    /// |win|USER
    Win(String),

    /// |tie
    Tie,

    // ===================
    // Major Actions
    // ===================
    /// |move|POKEMON|MOVE|TARGET
    Move {
        pokemon: Pokemon,
        move_name: String,
        target: Option<Pokemon>,
        miss: bool,
        still: bool,
        anim: Option<String>,
    },

    /// |switch|POKEMON|DETAILS|HP STATUS
    Switch {
        pokemon: Pokemon,
        details: PokemonDetails,
        hp_status: Option<HpStatus>,
    },

    /// |drag|POKEMON|DETAILS|HP STATUS
    Drag {
        pokemon: Pokemon,
        details: PokemonDetails,
        hp_status: Option<HpStatus>,
    },

    /// |detailschange|POKEMON|DETAILS|HP STATUS
    DetailsChange {
        pokemon: Pokemon,
        details: PokemonDetails,
        hp_status: Option<HpStatus>,
    },

    /// |-formechange|POKEMON|SPECIES|HP STATUS
    FormeChange {
        pokemon: Pokemon,
        species: String,
        hp_status: Option<HpStatus>,
    },

    /// |replace|POKEMON|DETAILS|HP STATUS
    Replace {
        pokemon: Pokemon,
        details: PokemonDetails,
        hp_status: Option<HpStatus>,
    },

    /// |swap|POKEMON|POSITION
    Swap { pokemon: Pokemon, position: u8 },

    /// |cant|POKEMON|REASON|MOVE?
    Cant {
        pokemon: Pokemon,
        reason: String,
        move_name: Option<String>,
    },

    /// |faint|POKEMON
    Faint(Pokemon),

    // ===================
    // Minor Actions
    // ===================
    /// |-fail|POKEMON|ACTION?
    Fail {
        pokemon: Pokemon,
        action: Option<String>,
    },

    /// |-block|POKEMON|EFFECT|MOVE?|ATTACKER?
    Block {
        pokemon: Pokemon,
        effect: String,
        move_name: Option<String>,
        attacker: Option<Pokemon>,
    },

    /// |-notarget|POKEMON?
    NoTarget(Option<Pokemon>),

    /// |-miss|SOURCE|TARGET?
    Miss {
        source: Pokemon,
        target: Option<Pokemon>,
    },

    /// |-damage|POKEMON|HP STATUS
    Damage {
        pokemon: Pokemon,
        hp_status: Option<HpStatus>,
    },

    /// |-heal|POKEMON|HP STATUS
    Heal {
        pokemon: Pokemon,
        hp_status: Option<HpStatus>,
    },

    /// |-sethp|POKEMON|HP
    SetHp { pokemon: Pokemon, hp_status: Option<HpStatus> },

    /// |-status|POKEMON|STATUS
    Status { pokemon: Pokemon, status: String },

    /// |-curestatus|POKEMON|STATUS
    CureStatus { pokemon: Pokemon, status: String },

    /// |-cureteam|POKEMON
    CureTeam(Pokemon),

    /// |-boost|POKEMON|STAT|AMOUNT
    Boost {
        pokemon: Pokemon,
        stat: Stat,
        amount: i8,
    },

    /// |-unboost|POKEMON|STAT|AMOUNT
    Unboost {
        pokemon: Pokemon,
        stat: Stat,
        amount: i8,
    },

    /// |-setboost|POKEMON|STAT|AMOUNT
    SetBoost {
        pokemon: Pokemon,
        stat: Stat,
        amount: i8,
    },

    /// |-swapboost|SOURCE|TARGET|STATS
    SwapBoost {
        source: Pokemon,
        target: Pokemon,
        stats: Vec<Stat>,
    },

    /// |-invertboost|POKEMON
    InvertBoost(Pokemon),

    /// |-clearboost|POKEMON
    ClearBoost(Pokemon),

    /// |-clearallboost
    ClearAllBoost,

    /// |-clearpositiveboost|TARGET|POKEMON|EFFECT
    ClearPositiveBoost {
        target: Pokemon,
        source: Pokemon,
        effect: String,
    },

    /// |-clearnegativeboost|POKEMON
    ClearNegativeBoost(Pokemon),

    /// |-copyboost|SOURCE|TARGET
    CopyBoost { source: Pokemon, target: Pokemon },

    /// |-weather|WEATHER
    Weather { weather: String, upkeep: bool },

    /// |-fieldstart|CONDITION
    FieldStart(String),

    /// |-fieldend|CONDITION
    FieldEnd(String),

    /// |-sidestart|SIDE|CONDITION
    SideStart { side: Side, condition: String },

    /// |-sideend|SIDE|CONDITION
    SideEnd { side: Side, condition: String },

    /// |-swapsideconditions
    SwapSideConditions,

    /// |-start|POKEMON|EFFECT
    VolatileStart { pokemon: Pokemon, effect: String },

    /// |-end|POKEMON|EFFECT
    VolatileEnd { pokemon: Pokemon, effect: String },

    /// |-crit|POKEMON
    Crit(Pokemon),

    /// |-supereffective|POKEMON
    SuperEffective(Pokemon),

    /// |-resisted|POKEMON
    Resisted(Pokemon),

    /// |-immune|POKEMON
    Immune(Pokemon),

    /// |-item|POKEMON|ITEM
    Item {
        pokemon: Pokemon,
        item: String,
        from: Option<String>,
    },

    /// |-enditem|POKEMON|ITEM
    EndItem {
        pokemon: Pokemon,
        item: String,
        from: Option<String>,
        eat: bool,
    },

    /// |-ability|POKEMON|ABILITY
    Ability {
        pokemon: Pokemon,
        ability: String,
        from: Option<String>,
    },

    /// |-endability|POKEMON
    EndAbility(Pokemon),

    /// |-transform|POKEMON|SPECIES
    Transform { pokemon: Pokemon, species: String },

    /// |-mega|POKEMON|MEGASTONE
    Mega { pokemon: Pokemon, megastone: String },

    /// |-primal|POKEMON
    Primal(Pokemon),

    /// |-burst|POKEMON|SPECIES|ITEM
    Burst {
        pokemon: Pokemon,
        species: String,
        item: String,
    },

    /// |-zpower|POKEMON
    ZPower(Pokemon),

    /// |-zbroken|POKEMON
    ZBroken(Pokemon),

    /// |-activate|EFFECT
    Activate {
        pokemon: Option<Pokemon>,
        effect: String,
    },

    /// |-hint|MESSAGE
    Hint(String),

    /// |-center
    Center,

    /// |-message|MESSAGE
    Message(String),

    /// |-combine
    Combine,

    /// |-waiting|SOURCE|TARGET
    Waiting { source: Pokemon, target: Pokemon },

    /// |-prepare|ATTACKER|MOVE|DEFENDER?
    Prepare {
        attacker: Pokemon,
        move_name: String,
        defender: Option<Pokemon>,
    },

    /// |-mustrecharge|POKEMON
    MustRecharge(Pokemon),

    /// |-nothing (deprecated)
    Nothing,

    /// |-hitcount|POKEMON|NUM
    HitCount { pokemon: Pokemon, count: u8 },

    /// |-singlemove|POKEMON|MOVE
    SingleMove { pokemon: Pokemon, move_name: String },

    /// |-singleturn|POKEMON|MOVE
    SingleTurn { pokemon: Pokemon, move_name: String },

    /// Raw message for catch-all
    Raw(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RoomType {
    Chat,
    Battle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Format {
    pub name: String,
    /// Format uses random/generated teams
    pub random_team: bool,
    /// Format is available on ladder (searching)
    pub search_show: bool,
    /// Format is available for challenging
    pub challenge_show: bool,
    /// Format is available for tournaments
    pub tournament_show: bool,
    /// Format uses level 50
    pub level_50: bool,
    /// Format is best of 3
    pub best_of: bool,
    /// Format has tera preview
    pub tera_preview: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FormatSection {
    pub column: u32,
    pub name: String,
    pub formats: Vec<Format>,
}

/// Current search state from |updatesearch|
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SearchState {
    /// Format IDs currently searching for
    #[serde(default)]
    pub searching: Vec<String>,
    /// Games currently in: room_id -> title (null if no games)
    #[serde(default)]
    pub games: Option<HashMap<String, String>>,
}

/// Info about an outgoing challenge
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ChallengeInfo {
    /// User being challenged
    pub to: String,
    /// Format of the challenge
    pub format: String,
}

/// Current challenge state from |updatechallenges|
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeState {
    /// Incoming challenges: userid -> format
    #[serde(default)]
    pub challenges_from: HashMap<String, String>,
    /// Outgoing challenge (if any)
    #[serde(default)]
    pub challenge_to: Option<ChallengeInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerFrame {
    pub room_id: Option<String>,
    pub messages: Vec<ServerMessage>,
}

pub fn parse_server_frame(frame: &str) -> Result<ServerFrame> {
    let mut lines = frame.lines();
    let mut room_id = None;

    // Check if first line is >ROOMID
    if let Some(first_line) = lines.clone().next()
        && let Some(room) = first_line.strip_prefix('>') {
            room_id = Some(room.to_string());
            lines.next();
        }

    // Parse remaining lines as messages
    let messages: Vec<ServerMessage> = lines
        .filter(|line| !line.trim().is_empty())
        .map(parse_server_message)
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(ServerFrame { room_id, messages })
}

pub fn parse_server_message(line: &str) -> Result<ServerMessage> {
    let line = line.trim();

    if line.is_empty() {
        return Ok(ServerMessage::Raw(String::new()));
    }

    if !line.starts_with('|') {
        return Ok(ServerMessage::Raw(line.to_string()));
    }

    let parts: Vec<&str> = line.split('|').collect();

    if parts.len() < 2 {
        return Ok(ServerMessage::Raw(line.to_string()));
    }

    match parts[1] {
        // Global messages
        "challstr" => global::parse_challstr(&parts),
        "updateuser" => global::parse_updateuser(&parts),
        "nametaken" => global::parse_nametaken(&parts),
        "popup" => global::parse_popup(&parts),
        "pm" => global::parse_pm(&parts),
        "usercount" => global::parse_usercount(&parts),
        "formats" => global::parse_formats(&parts),
        "updatesearch" => global::parse_updatesearch(&parts),
        "updatechallenges" => global::parse_updatechallenges(&parts),

        // Room messages
        "join" | "j" => room::parse_join(&parts, false),
        "J" => room::parse_join(&parts, true),
        "leave" | "l" => room::parse_leave(&parts, false),
        "L" => room::parse_leave(&parts, true),
        "init" => room::parse_init(&parts),
        "title" => room::parse_title(&parts),
        "users" => room::parse_users(&parts),
        "chat" | "c" => room::parse_chat(&parts, None),
        "c:" => room::parse_timestamped_chat(&parts),
        ":" => room::parse_timestamp(&parts),
        "battle" | "b" => room::parse_battle(&parts),
        "notify" => room::parse_notify(&parts),
        "name" | "n" => room::parse_name(&parts, false),
        "N" => room::parse_name(&parts, true),
        "html" => room::parse_html(&parts),
        "uhtml" => room::parse_uhtml(&parts),
        "uhtmlchange" => room::parse_uhtmlchange(&parts),

        // Battle initialization
        "player" => battle_init::parse_player(&parts),
        "teamsize" => battle_init::parse_teamsize(&parts),
        "gametype" => battle_init::parse_gametype(&parts),
        "gen" => battle_init::parse_gen(&parts),
        "tier" => battle_init::parse_tier(&parts),
        "rated" => battle_init::parse_rated(&parts),
        "rule" => battle_init::parse_rule(&parts),
        "clearpoke" => battle_init::parse_clearpoke(&parts),
        "poke" => battle_init::parse_poke(&parts),
        "teampreview" => battle_init::parse_teampreview(&parts),
        "start" => battle_init::parse_start(&parts),

        // Battle progress
        "request" => battle_progress::parse_request(&parts),
        "inactive" => battle_progress::parse_inactive(&parts),
        "inactiveoff" => battle_progress::parse_inactiveoff(&parts),
        "upkeep" => battle_progress::parse_upkeep(&parts),
        "turn" => battle_progress::parse_turn(&parts),
        "win" => battle_progress::parse_win(&parts),
        "tie" => battle_progress::parse_tie(&parts),

        // Major actions
        "move" => battle_major::parse_move(&parts),
        "switch" => battle_major::parse_switch(&parts),
        "drag" => battle_major::parse_drag(&parts),
        "detailschange" => battle_major::parse_detailschange(&parts),
        "replace" => battle_major::parse_replace(&parts),
        "swap" => battle_major::parse_swap(&parts),
        "cant" => battle_major::parse_cant(&parts),
        "faint" => battle_major::parse_faint(&parts),

        // Minor actions (start with -)
        "-formechange" => battle_major::parse_formechange(&parts),
        "-fail" => battle_minor::parse_fail(&parts),
        "-block" => battle_minor::parse_block(&parts),
        "-notarget" => battle_minor::parse_notarget(&parts),
        "-miss" => battle_minor::parse_miss(&parts),
        "-damage" => battle_minor::parse_damage(&parts),
        "-heal" => battle_minor::parse_heal(&parts),
        "-sethp" => battle_minor::parse_sethp(&parts),
        "-status" => battle_minor::parse_status(&parts),
        "-curestatus" => battle_minor::parse_curestatus(&parts),
        "-cureteam" => battle_minor::parse_cureteam(&parts),
        "-boost" => battle_minor::parse_boost(&parts),
        "-unboost" => battle_minor::parse_unboost(&parts),
        "-setboost" => battle_minor::parse_setboost(&parts),
        "-swapboost" => battle_minor::parse_swapboost(&parts),
        "-invertboost" => battle_minor::parse_invertboost(&parts),
        "-clearboost" => battle_minor::parse_clearboost(&parts),
        "-clearallboost" => battle_minor::parse_clearallboost(&parts),
        "-clearpositiveboost" => battle_minor::parse_clearpositiveboost(&parts),
        "-clearnegativeboost" => battle_minor::parse_clearnegativeboost(&parts),
        "-copyboost" => battle_minor::parse_copyboost(&parts),
        "-weather" => battle_minor::parse_weather(&parts),
        "-fieldstart" => battle_minor::parse_fieldstart(&parts),
        "-fieldend" => battle_minor::parse_fieldend(&parts),
        "-sidestart" => battle_minor::parse_sidestart(&parts),
        "-sideend" => battle_minor::parse_sideend(&parts),
        "-swapsideconditions" => battle_minor::parse_swapsideconditions(&parts),
        "-start" => battle_minor::parse_start(&parts),
        "-end" => battle_minor::parse_end(&parts),
        "-crit" => battle_minor::parse_crit(&parts),
        "-supereffective" => battle_minor::parse_supereffective(&parts),
        "-resisted" => battle_minor::parse_resisted(&parts),
        "-immune" => battle_minor::parse_immune(&parts),
        "-item" => battle_minor::parse_item(&parts),
        "-enditem" => battle_minor::parse_enditem(&parts),
        "-ability" => battle_minor::parse_ability(&parts),
        "-endability" => battle_minor::parse_endability(&parts),
        "-transform" => battle_minor::parse_transform(&parts),
        "-mega" => battle_minor::parse_mega(&parts),
        "-primal" => battle_minor::parse_primal(&parts),
        "-burst" => battle_minor::parse_burst(&parts),
        "-zpower" => battle_minor::parse_zpower(&parts),
        "-zbroken" => battle_minor::parse_zbroken(&parts),
        "-activate" => battle_minor::parse_activate(&parts),
        "-hint" => battle_minor::parse_hint(&parts),
        "-center" => battle_minor::parse_center(&parts),
        "-message" => battle_minor::parse_message(&parts),
        "-combine" => battle_minor::parse_combine(&parts),
        "-waiting" => battle_minor::parse_waiting(&parts),
        "-prepare" => battle_minor::parse_prepare(&parts),
        "-mustrecharge" => battle_minor::parse_mustrecharge(&parts),
        "-nothing" => battle_minor::parse_nothing(&parts),
        "-hitcount" => battle_minor::parse_hitcount(&parts),
        "-singlemove" => battle_minor::parse_singlemove(&parts),
        "-singleturn" => battle_minor::parse_singleturn(&parts),

        _ => Ok(ServerMessage::Raw(line.to_string())),
    }
}
