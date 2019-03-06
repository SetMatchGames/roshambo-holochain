#![feature(try_from)]

#[macro_use]
extern crate hdk;
extern crate serde;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
extern crate holochain_core_types_derive;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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
};

// The raw move choice and nonce can be stored locally and initially hashed in a front-end, 
// since I believe holochain can only store data publicly right now.
// The front-end will pass that hash to the player's local chain, from which it can be shared with other players.
// Once both players have the other player's hash, the players can reveal the raw data and confirm it with the hash.
#[derive(Serialize, Deserialize, Debug, DefaultJson, Hash)]
pub enum ValidMove {
    Rock,
    Paper,
    Scissors,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Hash)]
pub struct MoveChoice {
    name: ValidMove,
    nonce: String,
    address: Address,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct MoveChoiceHash {
    hash: u64,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub enum RoundResult {
    Win(Address),
    Draw,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct GameResult {
    player_addresses: [Address; 2],
    result: RoundResult,
    // Add a time stamp here -- how to get the two players to agree on it?
        // Maybe there are only time stamps for moves, and the game time comes from those somehow. 
    // Add a game ID here -- how to get the two players to agree on it?
        // something related to a websocket handshake at the beginning?
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

        validation: |_my_entry: MoveChoiceHash, _validation_data: hdk::ValidationData| {
            Ok(())
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

        validation: |_my_entry: GameResult, _validation_data: hdk::ValidationData| {
            Ok(())
        }
    )
}

pub fn handle_commit_move_choice_hash(entry: MoveChoiceHash) -> ZomeApiResult<Address> {
    let entry = Entry::App("move_choice_hash".into(), entry.into());
    let address = hdk::commit_entry(&entry)?;
    Ok(address)
}

pub fn handle_get_entry(address: Address) -> ZomeApiResult<Option<Entry>> {
    hdk::get_entry(&address)
}

// functions still needed:
    // resolve two choices into a game result (once their raw data is revealed and confirmed)

pub fn handle_confirm_choices_and_create_game_result(p1move: MoveChoice, 
    p2move: MoveChoice, p1hash_address: Address, 
    p2hash_address: Address) -> GameResult {
    // the idea here is to make it impossible for either player 
    // to create a game result before both move choice hashes have been committed
        // take in both player's raw move choices
        // hash and compare to the hashes previously committed
        // if those are equal, create the game result
    GameResult { player_addresses: [p1hash_address, p2hash_address], result: RoundResult::Draw, }
}

pub fn handle_commit_game_result(entry: GameResult) -> ZomeApiResult<Address> {
    let entry = Entry::App("game_result".into(), entry.into());
    let address = hdk::commit_entry(&entry)?;
    Ok(address)
}

define_zome! {
    entries: [
       define_move_choice_hash_entry(),
       define_game_result_entry()
    ]

    genesis: || { Ok(()) }

    functions: [
        commit_move_choice_hash: {
            inputs: |entry: MoveChoiceHash|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: handle_commit_move_choice_hash
        }
        commit_game_result: {
            inputs: |entry: GameResult|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: handle_commit_game_result
        }
        get_entry: {
            inputs: |address: Address|,
            outputs: |result: ZomeApiResult<Option<Entry>>|,
            handler: handle_get_entry
        }
    ]

    traits: {
        hc_public [
            create_move_choice_hash,
            create_game_result,
            get_entry
        ]
    }
}

// Helper functions

fn calculate_hash<T: Hash>(raw_data: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    raw_data.hash(&mut hasher);
    hasher.finish()
}

fn compare_move_choice_hashes(move_choice: MoveChoice, hash1: MoveChoiceHash) -> bool {
    let hash2 = MoveChoiceHash { hash: calculate_hash(&move_choice) };
    match hash1 {
        hash2 => true,
        _ => false,
    }
}

fn create_game_result(p1move: MoveChoice, p2move: MoveChoice) -> GameResult {
    let addresses: [Address; 2] = [ p1move.address.clone(), p2move.address.clone() ];
    let round_result: RoundResult = match p1move.name {
        ValidMove::Rock => match p2move.name {
                ValidMove::Paper => RoundResult::Win(p2move.address),
                ValidMove::Scissors => RoundResult::Win(p1move.address),
                ValidMove::Rock => RoundResult::Draw,
            },
        ValidMove::Paper => match p2move.name {
                ValidMove::Rock => RoundResult::Win(p1move.address),
                ValidMove::Scissors => RoundResult::Win(p2move.address),
                ValidMove::Paper => RoundResult::Draw,
            },
        ValidMove::Scissors => match p2move.name {
                ValidMove::Rock => RoundResult::Win(p2move.address),
                ValidMove::Paper => RoundResult::Win(p1move.address),
                ValidMove::Scissors => RoundResult::Draw,
            },
    };
    GameResult { player_addresses: addresses, result: round_result }
}