// import the rpc-websockets library
let WebSocket = require('rpc-websockets').Client

// instantiate Client and connect to an RPC server
let holochainUri = 'ws://localhost:4000'
let ws = new WebSocket(holochainUri)

// components
const rock = { "name": "Rock", "wins_against": ["Scissors"], "loses_against": ["Paper"] }
const paper = { "name": "Paper", "wins_against": ["Rock"], "loses_against": ["Scissors"] }
const scissors = { "name": "Scissors", "wins_against": ["Paper"], "loses_against": ["Rock"] }
const nonce = "nonce"
 
// create an event listener, and a callback, for when the socket connection opens
ws.on('open', async () => {
  // do stuff in here
  let method = 'info/instances'
  let params = {}
  // call an RPC method with parameters
  ws.call(method, params).then(result => {
      console.log(result)
  })
  
  
  method = 'roshambo instance 1/roshambo/new_offer'
  params = { "challenger_id_": "testId", "format_id_": "testFormat" }
  ws.call(method,params).then(r => { console.log(r)})
  /*
  let offerResult = await ws.call(method, params)
  console.log(offerResult)
  offerAddress = JSON.parse(offerResult).Ok
  console.log(offerAddress)
  /*
  method = 'roshambo instance 1/roshambo/new_commitment'
  params = { "component_": rock, "offer_address_": offerAddress, "host_id_": "fakeAddress", "nonce_": nonce }
  let commitmentResult = await ws.call(method, params)
  console.log(commitmentResult)
  */
  
})