#![feature(try_from)]

#[macro_use]
extern crate hdk;
extern crate serde;
extern crate multihash;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
extern crate holochain_core_types_derive;

use std::convert::TryInto;
use multihash::Hash as Multihash;

use hdk::{
    entry_definition::ValidatingEntryType,
    error::ZomeApiResult,
};

use hdk::holochain_core_types::{
    cas::content::Address, 
    entry::Entry, 
    dna::entry_types::Sharing, 
    error::HolochainError, 
    json::JsonString,
    agent::AgentId,
    hash::HashString,
};

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct GameId {
    player_1_address: AgentId,
    player_2_address: AgentId,
}
// should probably make sure both players share the exact same GameId entry (same address)

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct MoveChoiceHash {
    info_hash: HashString,
    game_id_address: Address,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub enum ValidMove {
    Rock,
    Paper,
    Scissors,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct MoveInfo {
    name: ValidMove,
    nonce: String,
    author: AgentId,
    opponent: AgentId,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct MoveChoice {
    info: MoveInfo,
    info_hash_address: Address,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct GameInfo {
    player_1_move_address: Address,
    player_2_move_address: Address,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub enum GameResult {
    Win { 
        winner_address: AgentId, 
        loser_address: AgentId, 
        info: GameInfo,
    },
    Draw { 
        player_1_address: AgentId, 
        player_2_address: AgentId, 
        info: GameInfo,
    },
}

fn define_game_id_entry() -> ValidatingEntryType {
    entry!(
        name: "game_id",
        description: "holds this game's player addresses at a unique address",
        sharing: Sharing::Public,
        native_type: GameId,
        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },
        validation: |_game_id: GameId, _validation_data: hdk::ValidationData| {
            // ensures the players share the same game_id?
            Ok(())
        }
    )
}

fn define_move_choice_hash_entry() -> ValidatingEntryType {
    entry!(
        name: "move_choice_hash",
        description: "encrypted version of a player's move choice",
        sharing: Sharing::Public,
        native_type: MoveChoiceHash,
        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },
        validation: |_hash: MoveChoiceHash, _validation_data: hdk::ValidationData| {
            // confirms that a game_id is at the address provided
            match handle_get_game_id(_hash.game_id_address) {
                Ok(_game_id) => Ok(()),
                _ => Err(String::from("No GameId at the address provided.")),
            }
            // to add: confirms that there is no other MoveChoiceHash submitted by this player for this game?
        }
    )
}

fn define_move_choice_entry() -> ValidatingEntryType {
    entry!(
        name: "move_choice",
        description: "a player's move choice",
        sharing: Sharing::Public,
        native_type: MoveChoice,
        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },
        validation: |_move: MoveChoice, _validation_data: hdk::ValidationData| {
            // returns Ok only if the hash at the address matches the hashed move info.
            let hash_check = calculate_hash(&_move.info);
            match handle_get_move_choice_hash(_move.info_hash_address) {
                Ok(MoveChoiceHash{info_hash,game_id_address:_}) => if hash_check == info_hash {
                        Ok(())
                    } else {
                        Err(String::from("Move choice hashes do not match."))
                    },
                _ => Err(String::from("No MoveChoiceHash at the address provided.")),
            }
        }
    )
}

fn define_game_result_entry() -> ValidatingEntryType {
    entry!(
        name: "game_result",
        description: "results of a game of Roshambo",
        sharing: Sharing::Public,
        native_type: GameResult,
        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },
        validation: |_game_result: GameResult, _validation_data: hdk::ValidationData| {
            // returns Ok if 
                // the player addresses match the move addresses
                // AND the game id addresses of the info hashes of the moves match 
                // AND the move choices resolve to the correct result
            match _game_result {
                GameResult::Win{winner_address,loser_address,info} => {
                    let p1move: MoveChoice = handle_get_move_choice(info.player_1_move_address.clone()).unwrap();
                    let p2move: MoveChoice = handle_get_move_choice(info.player_2_move_address.clone()).unwrap();

                    let p1move_info_hash: MoveChoiceHash = handle_get_move_choice_hash(p1move.info_hash_address.clone()).unwrap();
                    let p2move_info_hash: MoveChoiceHash = handle_get_move_choice_hash(p2move.info_hash_address.clone()).unwrap();

                    if (p1move.info.author == winner_address && p2move.info.author == loser_address) ||
                        (p1move.info.author == loser_address && p2move.info.author == winner_address) {
                        if p1move_info_hash.game_id_address == p2move_info_hash.game_id_address {
                            let address_to_check = &winner_address;
                            let test_result = handle_create_game_result(info.player_1_move_address, info.player_2_move_address).unwrap();
                            match test_result {
                                GameResult::Win{winner_address,loser_address:_,info:_} => if &winner_address == address_to_check {
                                        Ok(())
                                    } else {
                                        Err(String::from("Could not duplicate game result winner."))
                                    }
                                _ => Err(String::from("Could not duplicate game result."))
                            }
                        } else {
                            Err(String::from("GameIds do not match."))
                        }
                    } else {
                        Err(String::from("Move authors do not match game players."))
                    }
                },
                GameResult::Draw{player_1_address,player_2_address,info} => {
                    let p1move: MoveChoice = handle_get_move_choice(info.player_1_move_address.clone()).unwrap();
                    let p2move: MoveChoice = handle_get_move_choice(info.player_2_move_address.clone()).unwrap();

                    let p1move_info_hash: MoveChoiceHash = handle_get_move_choice_hash(p1move.info_hash_address.clone()).unwrap();
                    let p2move_info_hash: MoveChoiceHash = handle_get_move_choice_hash(p2move.info_hash_address.clone()).unwrap();

                    if p1move.info.author == player_1_address && p2move.info.author == player_2_address {
                        if p1move_info_hash.game_id_address == p2move_info_hash.game_id_address {
                            let address_to_check = &player_1_address;
                            let test_result = handle_create_game_result(info.player_1_move_address, info.player_2_move_address).unwrap();
                            match test_result {
                                GameResult::Draw{player_1_address,player_2_address:_,info:_} => if &player_1_address == address_to_check {
                                        Ok(())
                                    } else {
                                        Err(String::from("Could not duplicate game result winner."))
                                    }
                                _ => Err(String::from("Could not duplicate game result."))
                            }
                        } else {
                            Err(String::from("GameIds do not match."))
                        }
                    } else {
                        Err(String::from("Move authors do not match game players."))
                    }
                },
            }
        }
    )
}

pub fn handle_commit_game_id(entry: GameId) -> ZomeApiResult<Address> {
    let entry = Entry::App("game_id".into(), entry.into());
    let address = hdk::commit_entry(&entry)?;
    Ok(address)
}

pub fn handle_commit_move_choice_hash(entry: MoveChoiceHash) -> ZomeApiResult<Address> {
    let entry = Entry::App("move_choice_hash".into(), entry.into());
    let address = hdk::commit_entry(&entry)?;
    Ok(address)
}

pub fn handle_commit_move_choice(entry: MoveChoice) -> ZomeApiResult<Address> {
    let entry = Entry::App("move_choice".into(), entry.into());
    let address = hdk::commit_entry(&entry)?;
    Ok(address)
}

pub fn handle_create_game_result(p1move_address: Address, p2move_address: Address) -> Result<GameResult, String> {
    let game_info: GameInfo = GameInfo { player_1_move_address: p1move_address.clone(), player_2_move_address: p2move_address.clone() };

    let p1move: MoveChoice = handle_get_move_choice(p1move_address).unwrap();
    let p2move: MoveChoice = handle_get_move_choice(p2move_address).unwrap();

    resolve_moves_to_game_result(p1move, p2move, game_info)
}

pub fn handle_commit_game_result(entry: GameResult) -> ZomeApiResult<Address> {
    let entry = Entry::App("game_result".into(), entry.into());
    let address = hdk::commit_entry(&entry)?;
    Ok(address)
}

pub fn handle_get_game_id(address: Address) -> ZomeApiResult<GameId> {
    match hdk::get_entry(&address) {
        Ok(Some(Entry::App(_, api_result))) => Ok(api_result.try_into()?),
        _ => Err(String::from("No element found").into())
    }
}

pub fn handle_get_move_choice_hash(address: Address) -> ZomeApiResult<MoveChoiceHash> {
    match hdk::get_entry(&address) {
        Ok(Some(Entry::App(_, api_result))) => Ok(api_result.try_into()?),
        _ => Err(String::from("No element found").into())
    }
}

pub fn handle_get_move_choice(address: Address) -> ZomeApiResult<MoveChoice> {
    match hdk::get_entry(&address) {
        Ok(Some(Entry::App(_, api_result))) => Ok(api_result.try_into()?),
        _ => Err(String::from("No element found").into())
    }
}

pub fn handle_get_game_result(address: Address) -> ZomeApiResult<GameResult> {
    match hdk::get_entry(&address) {
        Ok(Some(Entry::App(_, api_result))) => Ok(api_result.try_into()?),
        _ => Err(String::from("No element found").into())
    }
}

define_zome! {
    entries: [
        define_game_id_entry(),
        define_move_choice_hash_entry(),
        define_move_choice_entry(),
        define_game_result_entry()
    ]

    genesis: || { Ok(()) }

    functions: [
        commit_game_id: {
            inputs: |entry: GameId|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: handle_commit_game_id
        }
        commit_move_choice_hash: {
            inputs: |entry: MoveChoiceHash|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: handle_commit_move_choice_hash
        }
        commit_move_choice: {
            inputs: |entry: MoveChoice|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: handle_commit_move_choice
        }
        create_game_result: {
            inputs: |entry: GameResult|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: handle_commit_game_result
        }
        commit_game_result: {
            inputs: |entry: GameResult|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: handle_commit_game_result
        }
        get_game_id: {
            inputs: |address: Address|,
            outputs: |result: ZomeApiResult<GameId>|,
            handler: handle_get_game_id
        }
        get_move_choice_hash: {
            inputs: |address: Address|,
            outputs: |result: ZomeApiResult<MoveChoiceHash>|,
            handler: handle_get_move_choice_hash
        }
        get_move_choice: {
            inputs: |address: Address|,
            outputs: |result: ZomeApiResult<MoveChoice>|,
            handler: handle_get_move_choice
        }
        get_game_result: {
            inputs: |address: Address|,
            outputs: |result: ZomeApiResult<GameResult>|,
            handler: handle_get_game_result
        }
    ]

    traits: {
        hc_public [
            create_move_choice_hash,
            create_game_result
        ]
    }
}

// Helper functions

fn calculate_hash<T: Into<JsonString>>(raw_data: T) -> HashString {
    HashString::encode_from_json_string(raw_data.into(), Multihash::SHA2256)
}

fn resolve_moves_to_game_result(p1move: MoveChoice, p2move: MoveChoice, game_info: GameInfo) -> Result<GameResult, String> {
    match p1move.info.name {
        ValidMove::Rock => match p2move.info.name {
                ValidMove::Paper => Ok(GameResult::Win { winner_address: p2move.info.author, loser_address: p1move.info.author, info: game_info }),
                ValidMove::Scissors => Ok(GameResult::Win { winner_address: p1move.info.author, loser_address: p2move.info.author, info: game_info }),
                ValidMove::Rock => Ok(GameResult::Draw { player_1_address: p1move.info.author, player_2_address: p2move.info.author, info: game_info }),
            },
        ValidMove::Paper => match p2move.info.name {
                ValidMove::Rock => Ok(GameResult::Win { winner_address: p1move.info.author, loser_address: p2move.info.author, info: game_info }),
                ValidMove::Scissors => Ok(GameResult::Win { winner_address: p2move.info.author, loser_address: p1move.info.author, info: game_info }),
                ValidMove::Paper => Ok(GameResult::Draw { player_1_address: p1move.info.author, player_2_address: p2move.info.author, info: game_info }),
            },
        ValidMove::Scissors => match p2move.info.name {
                ValidMove::Rock => Ok(GameResult::Win { winner_address: p2move.info.author, loser_address: p1move.info.author, info: game_info }),
                ValidMove::Paper => Ok(GameResult::Win { winner_address: p1move.info.author, loser_address: p2move.info.author, info: game_info }),
                ValidMove::Scissors => Ok(GameResult::Draw { player_1_address: p1move.info.author, player_2_address: p2move.info.author, info: game_info }),
            },
    }
}