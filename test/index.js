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

scenario.runTape("a round of Roshambo can be played securely", (t, { alice, bob }) => {
  // test data

  // tests
  console.log(alice)
  console.log(bob)
  const offerAddress = alice.call("roshambo", "new_offer", { challenger_id_: bob.agentId, format_id_: "format id string" } ).Ok
  console.log("offer address:", offerAddress)
  
  const offer = alice.call("roshambo", "get_offer", { address: offerAddress } ).Ok
  console.log("offer:", offer)
  
  const rock = { name: "Rock", wins_against: ["Scissors"], loses_against: ["Paper"] }

  const commitmentAddress = bob.call("roshambo", "new_commitment", { component_: rock, offer_address_: offerAddress, host_id_: alice.agentId } )
  console.log("commitment address:", commitmentAddress)

  // const commitment = bob.call("roshambo", "get_commitment", { address: commitmentAddress } )
  // console.log("commitment:", commitment)


  // t.deepEqual(commitment, { Ok: { App: [ commitment ] } })
})