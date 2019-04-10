// This test file uses the tape testing framework.
// To learn more, go here: https://github.com/substack/tape
const { Config, Scenario } = require("@holochain/holochain-nodejs")
Scenario.setTape(require("tape"))

const dnaPath = "./dist/roshambo-holochain.dna.json"
const dna = Config.dna(dnaPath)

const agentAlice = Config.agent("alice")
const agentBob = Config.agent("bob")
const agentCharlie = Config.agent("charlie")
const instanceAlice = Config.instance(agentAlice, dna)
const instanceBob = Config.instance(agentBob, dna)
const instanceCharlie = Config.instance(agentCharlie, dna)
const scenario = new Scenario([instanceAlice, instanceBob, instanceCharlie], /*{debugLog: true}*/)

// test data
const rock = { name: "Rock", wins_against: ["Scissors"], loses_against: ["Paper"] }
const paper = { name: "Paper", wins_against: ["Rock"], loses_against: ["Scissors"] }
const scissors = { name: "Scissors", wins_against: ["Paper"], loses_against: ["Rock"] }
const nonceString = "random"

const inputSets = [
  {
    testDescription: "Successful round",
    offerData: {
      challenger_id_: "correct",
      format_id_: "format"
    },
    commitmentData: {
      component_: rock,
      offer_address_: "",
      host_id_: "correct",
      nonce_: nonceString
    },
    moveData: {
      component_: paper,
      commitment_address_: "",
      challenger_id_: "correct"
    },
    gameResultData: {
      reveal: { component: rock, nonce: nonceString},
      move_address: "",
      host_id: "correct"
    },
    result: "Ok"
  },
  {
    testDescription: "Wrong commitment host",
    offerData: {
      challenger_id_: "correct",
      format_id_: "format"
    },
    commitmentData: {
      component_: rock,
      offer_address_: "wrong",
      host_id_: "",
      nonce_: nonceString
    },
    moveData: {
      component_: paper,
      commitment_address_: "",
      challenger_id_: "correct"
    },
    gameResultData: {
      reveal: { component: rock, nonce: nonceString},
      move_address: "",
      host_id: "correct"
    },
    result: "SerializationError"
  },
  {
    testDescription: "Wrong move challenger",
    offerData: {
      challenger_id_: "correct",
      format_id_: "format"
    },
    commitmentData: {
      component_: rock,
      offer_address_: "correct",
      host_id_: "",
      nonce_: nonceString
    },
    moveData: {
      component_: paper,
      commitment_address_: "",
      challenger_id_: "wrong"
    },
    gameResultData: {
      reveal: { component: rock, nonce: nonceString},
      move_address: "",
      host_id: "correct"
    },
    result: "SerializationError"
  },
  {
    testDescription: "Wrong game result host",
    offerData: {
      challenger_id_: "correct",
      format_id_: "format"
    },
    commitmentData: {
      component_: rock,
      offer_address_: "correct",
      host_id_: "",
      nonce_: nonceString
    },
    moveData: {
      component_: paper,
      commitment_address_: "",
      challenger_id_: "correct"
    },
    gameResultData: {
      reveal: { component: rock, nonce: nonceString},
      move_address: "",
      host_id: "wrong"
    },
    result: "SerializationError"
  },
  {
    testDescription: "Wrong commitment author",
    offerData: {
      challenger_id_: "correct",
      format_id_: "format"
    },
    commitmentData: {
      component_: rock,
      offer_address_: "correct",
      host_id_: "",
      nonce_: nonceString
    },
    moveData: {
      component_: paper,
      commitment_address_: "",
      challenger_id_: "correct"
    },
    gameResultData: {
      reveal: { component: rock, nonce: nonceString},
      move_address: "",
      host_id: "correct"
    },
    commitmentAuthor: "charlie",
    result: "SerializationError"
  },
  {
    testDescription: "Wrong move author",
    offerData: {
      challenger_id_: "correct",
      format_id_: "format"
    },
    commitmentData: {
      component_: rock,
      offer_address_: "correct",
      host_id_: "",
      nonce_: nonceString
    },
    moveData: {
      component_: paper,
      commitment_address_: "",
      challenger_id_: "correct"
    },
    gameResultData: {
      reveal: { component: rock, nonce: nonceString},
      move_address: "",
      host_id: "correct"
    },
    moveAuthor: "charlie",
    result: "SerializationError"
  },
  {
    testDescription: "Wrong game result author",
    offerData: {
      challenger_id_: "correct",
      format_id_: "format"
    },
    commitmentData: {
      component_: rock,
      offer_address_: "correct",
      host_id_: "",
      nonce_: nonceString
    },
    moveData: {
      component_: paper,
      commitment_address_: "",
      challenger_id_: "correct"
    },
    gameResultData: {
      reveal: { component: rock, nonce: nonceString},
      move_address: "",
      host_id: "correct"
    },
    gameResultAuthor: "charlie",
    result: "SerializationError"
  },
  {
    testDescription: "Wrong reveal",
    offerData: {
      challenger_id_: "correct",
      format_id_: "format"
    },
    commitmentData: {
      component_: rock,
      offer_address_: "correct",
      host_id_: "",
      nonce_: nonceString
    },
    moveData: {
      component_: paper,
      commitment_address_: "",
      challenger_id_: "correct"
    },
    gameResultData: {
      reveal: { component: rock, nonce: "wrong"},
      move_address: "",
      host_id: "correct"
    },
    result: "SerializationError"
  },
]

// test function
const runTest = async (inputSet) => {
  scenario.runTape(inputSet.testDescription, async (t, { alice, bob, charlie }) => {
    if(inputSet.offerData.challenger_id_ == "correct") {
      inputSet.offerData.challenger_id_ = bob.agentId
    }
    const offerAddress = await alice.callSync("roshambo", "new_offer", inputSet.offerData)
  
    if(inputSet.commitmentData.host_id_ == "correct") {
      inputSet.commitmentData.host_id_ = alice.agentId
    }
    inputSet.commitmentData.offer_address_ = offerAddress.Ok
    if(inputSet.commitmentAuthor == "charlie") {
      commitmentAddress = await charlie.callSync("roshambo", "new_commitment", inputSet.commitmentData)
    } else {
      commitmentAddress = await bob.callSync("roshambo", "new_commitment", inputSet.commitmentData)
    }

    if(inputSet.moveData.challenger_id_ == "correct") {
      inputSet.moveData.challenger_id_ = bob.agentId
    }
    inputSet.moveData.commitment_address_ = commitmentAddress.Ok
    if(inputSet.moveAuthor == "charlie") {
      moveAddress = await charlie.callSync("roshambo", "new_move", inputSet.moveData)
    } else {
      moveAddress = await alice.callSync("roshambo", "new_move", inputSet.moveData)
    }
  
    if(inputSet.gameResultData.host_id == "correct") {
      inputSet.gameResultData.host_id = alice.agentId
    }
    inputSet.gameResultData.move_address = moveAddress.Ok
    if(inputSet.gameResultAuthor == "charlie") {
      gameResultAddress = await charlie.callSync("roshambo", "new_game_result", inputSet.gameResultData)
    } else {
      gameResultAddress = await bob.callSync("roshambo", "new_game_result", inputSet.gameResultData)
    }

    t.deepEqual(Object.keys(gameResultAddress)[0], inputSet.result)
  })
}

inputSets.forEach((inputSet) => { runTest(inputSet) })