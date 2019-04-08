// This test file uses the tape testing framework.
// To learn more, go here: https://github.com/substack/tape
const { Config, Scenario } = require("@holochain/holochain-nodejs")
Scenario.setTape(require("tape"))

const dnaPath = "./dist/roshambo.dna.json"
const dna = Config.dna(dnaPath)

const agentAlice = Config.agent("alice")
const agentBob = Config.agent("bob")
const instanceAlice = Config.instance(agentAlice, dna)
const instanceBob = Config.instance(agentBob, dna)
const scenario = new Scenario([instanceAlice, instanceBob], /*{debugLog: true}*/)

// test data
const rock = { name: "Rock", wins_against: ["Scissors"], loses_against: ["Paper"] }
const paper = { name: "Paper", wins_against: ["Rock"], loses_against: ["Scissors"] }
const scissors = { name: "Scissors", wins_against: ["Paper"], loses_against: ["Rock"] }
const nonceString = "random"

const inputSets = [
  // successful round
  {
    offerData: {
      challenger_id_: "",
      format_id_: "format"
    },
    commitmentData: {
      component_: rock,
      offer_address_: "",
      host_id_: "",
      nonce_: nonceString
    },
    moveData: {
      component_: paper,
      commitment_address_: "",
      challenger_id_: ""
    },
    gameResultData: {
      reveal: { component: rock, nonce: nonceString},
      move_address: "",
      host_id: ""
    },
    result: true
  },
]

// tests
scenario.runTape("a round of Roshambo can be played securely", async (t, { alice, bob }) => {

  /*
  const offerAddress = await alice.callSync("roshambo", "new_offer", { challenger_id_: bob.agentId, format_id_: "format id string" } )
  const commitmentAddress = await bob.callSync("roshambo", "new_commitment", { component_: rock, offer_address_: offerAddress.Ok, host_id_: alice.agentId, nonce_: "randomstring" } )
  const moveAddress = await alice.callSync("roshambo", "new_move", { component_: paper, commitment_address_: commitmentAddress.Ok, challenger_id_: bob.agentId } )

  const reveal_ = { component: rock, nonce: "randomstring"}

  const gameResultAddress = await bob.callSync("roshambo", "new_game_result", { reveal: reveal_, move_address: moveAddress.Ok, host_id: alice.agentId } )
  const gameResult = await bob.callSync("roshambo", "get_game_result", { address: gameResultAddress.Ok } )

  t.deepEqual(gameResult.Ok.Win.winner_id, alice.agentId)
  */
  
  inputSets.forEach(async (inputSet) => {
    const offerAddress = await alice.callSync("roshambo", "new_offer", inputSet.offerData )
  
    inputSet.commitmentData.offer_address_ = offerAddress
    const commitmentAddress = await bob.callSync("roshambo", "new_commitment", inputSet.commitmentData )
  
    inputSet.moveData.offer_address_ = commitmentAddress
    const moveAddress = await alice.callSync("roshambo", "new_move", inputSet.moveData )
  
    inputSet.gameResultData.offer_address_ = moveAddress
    const gameResultAddress = await bob.callSync("roshambo", "new_game_result", inputSet.gameResultData )
    const gameResult = await bob.callSync("roshambo", "get_game_result", { address: gameResultAddress.Ok } )

    t.deepEqual(gameResult.host_id, alice.agentId)
  })

})