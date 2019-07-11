use std::{thread, time};

use reqwest::{Client as HttpClient, Url};
use super::API_BASE_URL;
use serde::{Deserialize, Deserializer};
use websocket::{
    ClientBuilder as WsClientBuilder,
    client::sync::Client as WsClientSync,
    stream::sync::{
        TlsStream as WsTlsStream,
        TcpStream as WsTcpStream,
    },
    message as WsMessage,
};



#[derive(Deserialize,Debug)]
struct GatewayResponse {
    pub url: String,
    pub shards: u32,
    pub session_start_limit: GatewaySessionStartLimit,
}

#[derive(Deserialize,Debug)]
struct GatewaySessionStartLimit {
    pub total: u32,
    pub remaining: u32,
    pub reset_after: u32,
}

#[derive(Debug)]
struct GatewayPayload{
    pub op: i32, // Op-code
    pub d: GatewayPayloadData, 
    pub s: i32, // Sequence number
    pub t: String,  // Event name
}

#[derive(Debug)]
enum GatewayPayloadData {
    Hello(HelloMsg),
    _Empty,
}

// After gateway websocket connection is initiated a hello message is sent from server
#[derive(Deserialize,Debug)]
struct HelloMsg{
    // Client should send a heartbeat to server every <heartbeat_interval> milliseconds
    heartbeat_interval: u64,
}

// GatewayPayload contains both opcode and data where the format of data is dependant
// of the opcode. This is the custom intermediate deserialization that first extracts
// the data as raw json value and deserializes after exstracting the opcode
impl<'de> Deserialize<'de> for GatewayPayload {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> 
        where D: Deserializer<'de>, {
            #[derive(Deserialize)]
            struct Helper {
                op: i32,
                d: serde_json::Value,
                s: i32,
                t: String,
            }

            let helper = Helper::deserialize(deserializer)?;

            fn deserialize_payload_data<'a,T>(val: serde_json::Value) -> T 
                    where for<'de> T: serde::Deserialize<'de>{
                serde_json::from_value(val).unwrap()
            }

            let data = match helper.op {
                11 => GatewayPayloadData::Hello(deserialize_payload_data::<HelloMsg>(helper.d)),
                _ => {
                    panic!("Unknown discord gateway payload opcode");
                },
            };

            Ok(GatewayPayload{
                op: helper.op,
                d: data,
                s: helper.s,
                t: helper.t,
            })
    }
}


fn send_get(client: &HttpClient, url: &Url) -> String {
    let req = client.get(url.clone());
                            
    let mut resp = req.send().unwrap();
    let body = resp.text().unwrap();
    body
}

// General helping deserialization function. Used for reducing verbosity until 
// a more robust option is created
fn deserialize<'a, T: Deserialize<'a>>(body: &'a str) -> T {
    let v : T = match serde_json::from_str(body) {
        Err(e) => {
            println!("Something went wrong with deserializing json: {}", body);
            panic!(e);
        },
        Ok(a) => a,
    };
    v
}

// Creates and initiates connection for the websocket to the specified url and
// panics if it is unable to connect
fn create_websocket(url: &Url) -> WsClientSync<WsTlsStream<WsTcpStream>> {
    let ws_client = match WsClientBuilder::from_url(&url).connect_secure(None) {
        Err(e) => {
            println!("Error: {}", e);
            panic!(format!("Could not connect websocket to {}", url));
        }
        Ok(c) => c,
    };
    ws_client
}

// Parses a URL from a string and panics if it can't be parsed
fn create_url(url: &str) -> Url {
    let url: Url = match Url::parse(url) {
        Ok(v) => v,
        Err(_) => {
            panic!(format!("Can't parse websocket url: {}", url));
        },
    };
    url
}

// Blocks thread for param ms milliseconds
fn thread_sleep(time_ms: u64){
    let time_ms = time::Duration::from_millis(time_ms);
    thread::sleep(time_ms);
}

// Handles the custom blocking eventloop for a discord websocket gateway
fn gateway_eventloop_sync(
        client: &mut WsClientSync<WsTlsStream<WsTcpStream>>, 
        sleep_ms: u64
        ){
    loop {
        thread_sleep(sleep_ms);

        // Handle heartbeat   
        let resp: WsMessage::OwnedMessage = match client.recv_message(){
            Err(e) => {
                println!("Error with receiving message from websocket: {}", e);
                continue;
            },
            Ok(r) => r,
        };
        
        
    }
}

pub fn initiate_gateway(client: &HttpClient) -> bool{
    let url = create_url(&format!("{}gateway/bot", API_BASE_URL));

    let c = true;
    if c {
        let j = r#"{
                 "op": 11,
                 "d": {
                   "heartbeat_interval": 45000
                 },
                 "s":42,
                 "t":"Test-JSON"
               }"#;
        let c : GatewayPayload = serde_json::from_str(j).unwrap();
        println!("{:?}", c);
        return true;
    }

    //TODO add explicit version and encoding parameters to request
    let body = send_get(client, &url);

    let v : GatewayResponse = deserialize(&body);
    let mut ws = create_websocket(&create_url(&v.url));
    gateway_eventloop_sync(&mut ws, 500);
    
    println!("----\n{:?}\n------", v);
    true
}
