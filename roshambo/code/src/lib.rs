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
pub struct MoveChoice {
    name: String,
    nonce: String,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct MoveChoiceHash {
    hash: String,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct GameResult {
    player_addresses: Vec<Address>,
    winner_address: Option<Address>,
    loser_address: Option<Address>,
    draw: bool, 
    // Add a time stamp here -- how to get the two players to agree on it?
    // Add a game ID here -- how to get the two players to agree on it?
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

pub fn handle_commit_game_result(entry: GameResult) -> ZomeApiResult<Address> {
    let entry = Entry::App("game_result".into(), entry.into());
    let address = hdk::commit_entry(&entry)?;
    Ok(address)
}

pub fn handle_get_entry(address: Address) -> ZomeApiResult<Option<Entry>> {
    hdk::get_entry(&address)
}

// functions still needed:
    // resolve two choices into a game result (once their raw data is revealed and confirmed)
    // compare game results before committing

pub fn handle_compare_move_choice_hashes(move_choice: MoveChoice, hash1: MoveChoiceHash) -> bool {
    let hash2 = calculate_hash(move_choice)?;
    match hash1 {
        hash2 => true,
        _     => false
    }
}

pub fn resolve_move_choices(move1: MoveChoice, move2: MoveChoice) -> GameResult {

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
            handler: handle_create_move_choice_hash
        }
        commit_game_result: {
            inputs: |entry: GameResult|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: handle_create_game_result
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

fn calculate_hash<M: Hash>(raw_data: &M) -> u64 {
    let mut hasher = DefaultHasher::new();
    raw_data.hash(&mut hasher);
    hasher.finish()
}