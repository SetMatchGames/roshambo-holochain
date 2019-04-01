#![feature(try_from)]

#[macro_use]
extern crate hdk;
extern crate serde;
extern crate multihash;
extern crate rand;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
extern crate holochain_core_types_derive;

use multihash::Hash as Multihash;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use std::convert::TryInto;

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
    // agent::AgentId,
    hash::HashString,
    chain_header::ChainHeader,
};

use holochain_wasm_utils::api_serialization::get_entry::{
    GetEntryResultType,
    GetEntryOptions,
};


// each entry should be a state machine in the game
// each entry fully validates itself using previous one so no further validation down-chain is needed
// protocol:
    // 1. identify an opponent (entry w/ address)
    // 2. opponent commits a move hash (implicitly confirms the game is happening)
    // 3. player commits a move
    // 4. opponent commits a game result

// Questions
    // How do we get AgentIds? Is it the result if you get what's at the agent's address?
    // Should the format be stored on holochain so we can use format_address instead of format_id?
    // We should probably also be checking that the host and challenger are not the same person

// Types
/*
#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub enum Player {
    Host(Address),
    Challenger(Address),
}

impl PartialEq for Player {
    fn eq(&self, other: &Player) -> bool {
        match self {
            Player::Host(address) => match other {
                Player::Host(other_address) => address == other_address,
                _ => false,
            },
            Player::Challenger(address) => match other {
                Player::Challenger(other_address) => address == other_address,
                _ => false,
            },
        }
    }
}
*/

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct Offer {
    challenger_id: Address,
    format_id: String,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct Commitment {
    hash: HashString,
    offer_address: Address,
    host_id: Address,
    format_id: String,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct Component {
    name: String,
    wins_against: Vec<String>,
    loses_against: Vec<String>,
}

impl PartialEq for Component {
    fn eq(&self, other: &Component) -> bool {
        self.name == other.name &&
        self.wins_against == other.wins_against &&
        self.loses_against == other.loses_against
    }
}

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct Format {
    moves: Vec<Component>,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct Move {
    component: Component,
    commitment_address: Address,
    challenger_id: Address,
    hash: HashString,
    format_id: String,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct Reveal {
    component: Component,
    nonce: String,
}

impl PartialEq for Reveal {
    fn eq(&self, other: &Reveal) -> bool {
        self.component == other.component &&
        self.nonce == other.nonce
    }
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub enum GameResult {
    Win {
        reveal: Reveal,
        move_address: Address,
        winner_id: Address,
        loser_id: Address,
        format_id: String,
    },
    Draw {
        reveal: Reveal,
        move_address: Address,
        players: Vec<Address>,
        format_id: String,
    },
}

impl PartialEq for GameResult {
    fn eq(&self, other: &GameResult) -> bool {
        match self {
            GameResult::Win {
                reveal,
                move_address,
                winner_id,
                loser_id,
                format_id,
            } => {
                match other {
                    GameResult::Win {
                        reveal: other_reveal,
                        move_address: other_move_address,
                        winner_id: other_winner_id,
                        loser_id: other_loser_id,
                        format_id: other_format_id,
                    } => {
                        reveal == other_reveal &&
                        move_address == other_move_address &&
                        winner_id == other_winner_id && 
                        loser_id == other_loser_id &&
                        format_id == other_format_id
                    },
                    _ => false,
                }
            },
            GameResult::Draw {
                reveal,
                move_address,
                players,
                format_id,
            } => {
                match other {
                    GameResult::Draw {
                        reveal: other_reveal,
                        move_address: other_move_address,
                        players: other_players,
                        format_id: other_format_id,
                    } => {
                        reveal == other_reveal &&
                        move_address == other_move_address &&
                        players == other_players &&
                        format_id == other_format_id
                    },
                    _ => false,
                }
            },
        }
    }
}

// Entry definitions

fn define_offer_entry() -> ValidatingEntryType {
    entry!(
        name: "offer",
        description: "host agent offers to start a game with a challenger agent",
        sharing: Sharing::Public,
        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },
        validation: |validation_data: hdk::EntryValidationData<Offer>| {
            // challenger_id is an agent
            /*
            match offer.challenger_id {
                Player::Challenger(agent_id) => { handle_get_agent_id(agent_id.key.into())?; Ok(()) },
                _ => Err(String::from("No challenger").into()),
            }
            */
            if let hdk::EntryValidationData::Create{entry: offer, validation_data: _} = validation_data {
                hdk::debug(format!("validation: {} {}", offer.challenger_id, offer.format_id));
            } else { 
                hdk::debug(format!("could not destructure validation data")); 
            }
            Ok(())
        }
    )
}

fn define_commitment_entry() -> ValidatingEntryType {
    entry!(
        name: "commitment",
        description: "challenger agent accepts game offer by commiting a move hash",
        sharing: Sharing::Public,
        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },
        validation: |validation_data: hdk::EntryValidationData<Commitment>| {
            // offer.author == commitment.host, commitment.author == offer.challenger, format_ids match
            if let hdk::EntryValidationData::Create{entry: commitment, validation_data: validation_} = validation_data {
                let offer_author_address: Address = get_author(&commitment.offer_address)?.into();
                let offer: Offer = handle_get_offer(commitment.offer_address.clone())?;
                let commitment_author_address: Address = author_from_header(&validation_.package.chain_header)?;

                assert!(offer_author_address == commitment.host_id);
                assert!(commitment_author_address == offer.challenger_id);
                if commitment.format_id != offer.format_id {
                    return Err(String::from("Commitment format does not match offer").into());
                };
                Ok(())
            } else { Err(String::from("Unreachable").into()) }
        }
    )
}

fn define_move_entry() -> ValidatingEntryType {
    entry!(
        name: "move",
        description: "host submits their move in response to the challenger's hash",
        sharing: Sharing::Public,
        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },
        validation: |validation_data: hdk::EntryValidationData<Move>| {
            // move.author == commitment.host_id, challenger_id == commitment.author, move.hash == commitment.hash, 
            // move.component is in format, move.format_id == commitment.format_id
            if let hdk::EntryValidationData::Create{entry: move_, validation_data: validation_} = validation_data {
                let commitment_author_address: Address = get_author(&move_.commitment_address)?.into();
                let commitment: Commitment = handle_get_commitment(move_.commitment_address.clone())?;
                let move_author_address: Address = author_from_header(&validation_.package.chain_header)?;
                
                assert!(move_author_address == commitment.host_id);
                assert!(commitment_author_address == move_.challenger_id);
                if move_.hash != commitment.hash {
                    return Err(String::from("Move hash does not match commitment").into());
                };
                if move_.format_id != commitment.format_id {
                    return Err(String::from("Move format does not match commitment").into());
                };
                // TODO assert(_move.component is not in format);
                Ok(())
            } else { Err(String::from("Unreachable").into()) }
        }
    )
}

fn define_game_result_entry() -> ValidatingEntryType {
    entry!(
        name: "game_result",
        description: "challenger reveals a move that matches the hash commitment and commits a game result",
        sharing: Sharing::Public,
        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },
        validation: |validation_data: hdk::EntryValidationData<GameResult>| {
            // hash of reveal == move_.hash, reveal.component is in format, 
            // game_result.format_id == move.format_id
            if let hdk::EntryValidationData::Create{entry: game_result, validation_data: _} = validation_data {
                match game_result.clone() {
                    GameResult::Win {
                        reveal,
                        move_address,
                        winner_id: _, // validated by checking game result
                        loser_id: _,  // validated by checking game result
                        format_id,
                    } => validate_game_result(game_result, reveal, move_address, format_id),
                    GameResult::Draw {
                        reveal,
                        move_address,
                        players: _, // validated by checking game result
                        format_id,
                    } => validate_game_result(game_result, reveal, move_address, format_id),
                }
            } else { Err(String::from("Unreachable").into()) }
        }
    )
}

// Public functions

pub fn handle_new_offer(challenger_id_: Address, format_id_: String) -> ZomeApiResult<Address> {
    let offer = Offer {
        challenger_id: challenger_id_,
        format_id: format_id_,
    };

    let entry = Entry::App("offer".into(), offer.into());
    let address = hdk::commit_entry(&entry)?;
    Ok(address)
}

pub fn handle_new_commitment(component_: Component, offer_address_: Address, host_id_: Address) -> ZomeApiResult<Address> {
    let offer: Offer = handle_get_offer(offer_address_.clone())?;
    let nonce_: String = generate_nonce();
    let reveal = Reveal { component: component_, nonce: nonce_};
        // this reveal needs to get stored locally somehow (not available publicly on chain)
    let hashstring: HashString = calculate_hash(reveal);

    let commitment = Commitment {
        hash: hashstring,
        offer_address: offer_address_,
        host_id: host_id_,
        format_id: offer.format_id,
    };

    let entry = Entry::App("commitment".into(), commitment.into());
    let address = hdk::commit_entry(&entry)?;
    Ok(address)
}

pub fn handle_new_move(component_: Component, commitment_address_: Address, challenger_id_: Address) -> ZomeApiResult<Address> {
    let commitment: Commitment = handle_get_commitment(commitment_address_.clone())?;

    let move_ = Move {
        component: component_,
        commitment_address: commitment_address_,
        challenger_id: challenger_id_,
        hash: commitment.hash.clone(),
        format_id: commitment.format_id,
    };

    let entry = Entry::App("move".into(), move_.into());
    let address = hdk::commit_entry(&entry)?;
    Ok(address)
}

pub fn handle_new_game_result(reveal: Reveal, move_address: Address, host_id: Address) -> ZomeApiResult<Address> {
    let game_result: GameResult = create_game_result(reveal, move_address, host_id)?;

    let entry = Entry::App("game_result".into(), game_result.into());
    let address = hdk::commit_entry(&entry)?;
    Ok(address)
}

pub fn handle_get_offer(address: Address) -> ZomeApiResult<Offer> {
    match hdk::get_entry(&address) {
        Ok(Some(Entry::App(_, api_result))) => Ok(api_result.try_into()?),
        _ => Err(String::from("No offer found").into())
    }
}

pub fn handle_get_commitment(address: Address) -> ZomeApiResult<Commitment> {
    match hdk::get_entry(&address) {
        Ok(Some(Entry::App(_, api_result))) => Ok(api_result.try_into()?),
        _ => Err(String::from("No commitment found").into())
    }
}

pub fn handle_get_move(address: Address) -> ZomeApiResult<Move> {
    match hdk::get_entry(&address) {
        Ok(Some(Entry::App(_, api_result))) => Ok(api_result.try_into()?),
        _ => Err(String::from("No move found").into())
    }
}

/*
pub fn handle_get_game_result(address: Address) -> ZomeApiResult<GameResult> {
    match hdk::get_entry(&address) {
        Ok(Some(Entry::App(_, api_result))) => Ok(api_result.try_into()?),
        _ => Err(String::from("No game result found").into())
    }
}
*/

// this is not the correct way to handle agent ids
/*
pub fn handle_get_agent_id(address: Address) -> ZomeApiResult<AgentId> {
    if let Ok(Some(agent_id_entry)) = hdk::get_entry(&address) {
        match agent_id_entry {
            Entry::AgentId(agent_id) => return Ok(agent_id),
            _ => Err(String::from("Get AgentId failure: No agent found").into())
        }
    } else {
        Err(String::from("Get AgentId failure: No entry found at address").into())
    }
}
*/

define_zome! {
    entries: [
        define_offer_entry(),
        define_commitment_entry(),
        define_move_entry(),
        define_game_result_entry()
    ]

    genesis: || { Ok(()) }

    functions: [
        new_offer: {
            inputs: |challenger_id_: Address, format_id_: String|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: handle_new_offer
        }
        new_commitment: {
            inputs: |component_: Component, offer_address_: Address, host_id_: Address|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: handle_new_commitment
        }
        new_move: {
            inputs: |component_: Component, commitment_address_: Address, challenger_id_: Address|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: handle_new_move
        }
        new_game_result: {
            inputs: |reveal: Reveal, move_address: Address, host_id: Address|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: handle_new_game_result
        }
        get_offer: {
            inputs: |address: Address|,
            outputs: |result: ZomeApiResult<Offer>|,
            handler: handle_get_offer
        }
        get_commitment: {
            inputs: |address: Address|,
            outputs: |result: ZomeApiResult<Commitment>|,
            handler: handle_get_commitment
        }
        get_move: {
            inputs: |address: Address|,
            outputs: |result: ZomeApiResult<Move>|,
            handler: handle_get_move
        }
        /*
        get_agent_id: {
            inputs: |address: Address|,
            outputs: |result: ZomeApiResult<AgentId>|,
            handler: handle_get_agent_id
        }
        */
    ]

    traits: {
        hc_public [
            new_offer,
            get_offer,
            new_commitment,
            get_commitment,
            new_move,
            get_move,
            new_game_result
            // get_agent_id
        ]
    }
}

// Private helper functions

fn calculate_hash<T: Into<JsonString>>(raw_data: T) -> HashString {
    HashString::encode_from_json_string(raw_data.into(), Multihash::SHA2256)
}

fn get_author(entry_address: &Address) -> ZomeApiResult<Address> {
    if let GetEntryResultType::Single(result) = hdk::get_entry_result(
        entry_address,
        GetEntryOptions {
            entry: false,
            headers: true,
            ..Default::default()
        },
    )?
    .result
    {
        let author_address = result
            .headers
            .into_iter()
            .map(|header| author_from_header(&header).unwrap())
            .next()
            .unwrap();
        // this is not the correct way of handling agent ids
        // let agent_id: AgentId = handle_get_agent_id(author_address)?;
        return Ok(author_address);
    } else {
        unimplemented!()
    }
}

fn author_from_header(chain_header: &ChainHeader) -> ZomeApiResult<Address> {
    let author_address = chain_header.provenances()
        .first()
        .unwrap()
        .clone()
        .source();
    return Ok(author_address)
}

fn create_game_result(reveal_: Reveal, move_address_: Address, host_id: Address) -> ZomeApiResult<GameResult> {
    let move_: Move = handle_get_move(move_address_.clone())?;
    let host_component: &Component = &move_.component;
    let challenger_component: &Component = &reveal_.component;
    let format_id_ = move_.format_id.clone();
    let challenger_id = move_.challenger_id;
    let winner: String = resolve_components(host_component, challenger_component);

    if winner == String::from("host") {
        Ok(GameResult::Win {
            reveal: reveal_,
            move_address: move_address_,
            winner_id: host_id,
            loser_id: challenger_id,
            format_id: format_id_,
        })
    } else if winner == String::from("challenger") {
        Ok(GameResult::Win {
            reveal: reveal_,
            move_address: move_address_,
            winner_id: challenger_id,
            loser_id: host_id,
            format_id: format_id_,
        })
    } else if winner == String::from("draw") {
        Ok(GameResult::Draw {
            reveal: reveal_,
            move_address: move_address_,
            players: vec![host_id, challenger_id],
            format_id: format_id_,
        })
    } else {
        unimplemented!();
    }
}

fn resolve_components(host_component: &Component, challenger_component: &Component) -> String {
    if host_component.wins_against.contains(&challenger_component.name) {
        return String::from("host");
    }
    if challenger_component.wins_against.contains(&host_component.name) {
        return String::from("challenger");
    }
    if host_component.loses_against.contains(&challenger_component.name) {
        return String::from("challenger");
    }
    if challenger_component.loses_against.contains(&host_component.name) {
        return String::from("host");
    }
    return String::from("draw");
}

fn validate_game_result(game_result: GameResult, reveal: Reveal, move_address: Address, format_id: String) -> Result<(), String> {
    let move_author: Address = get_author(&move_address)?;
    let move_: Move = handle_get_move(move_address.clone())?;
    

    if move_.hash != calculate_hash(reveal.clone()) {
        return Err(String::from("Move hash does not match hash of reveal"));
    }
    if format_id != move_.format_id {
        return Err(String::from("Format id does not match move"));
    }
    if game_result != create_game_result(reveal, move_address, move_author)? {
        return Err(String::from("Game results do not match"));
    }

    // TODO Assert!(reveal component is in the format);

    Ok(())
}

fn generate_nonce() -> String {
    thread_rng() // TODO research cryptographic security of this RNG
        .sample_iter(&Alphanumeric)
        .take(24)
        .collect()
}