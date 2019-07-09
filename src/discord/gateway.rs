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

#[derive(Deserialize, Debug)]
struct GatewayPayload<T>{
    pub op: i32, // Op-code
    pub d: T, // Data message(unspecified format that is specified with op code) 
    pub s: i32, // Sequence number
    pub t: String,  // Event name
}

// After gateway websocket connection is initiated a hello message is sent from server
#[derive(Deserialize, Debug)]
struct HelloMsg{
    // Client should send a heartbeat to server every <heartbeat_interval> milliseconds
    heartbeat_interval: u64,
}

// TODO Fix deserialization for Gateway payload
// Problem: opcode decides generic type of data mid deserialization
// Idea: intermediate object that can be converted to real object, avoiding the subject
// of mid deserialization. Saving data as raw string and in convertion method deserializes
// to proper struct
//impl<'de> Deserialize<'de> for GatewayPayload {
//    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> 
//        where D: Deserializer<'de>, {
//        
//    }
//}


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

    //TODO add explicit version and encoding parameters to request
    let body = send_get(client, &url);

    let v : GatewayResponse = deserialize(&body);
    let mut ws = create_websocket(&create_url(&v.url));
    gateway_eventloop_sync(&mut ws, 500);
    
    println!("----\n{:?}\n------", v);
    true
}
