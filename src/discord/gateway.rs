use std::{thread, time};

use reqwest::{Client as HttpClient, Url};
use super::API_BASE_URL;
use serde::{de, Deserialize, Deserializer, Serialize};
use websocket::{
    ClientBuilder,
    client::sync::Client as WsClientSync,
    stream::sync::{
        TlsStream as WsTlsStreamSync,
        TcpStream as WsTcpStreamSync,
    },
    client::r#async::{
        Client,
        ClientNew,
        TlsStream,
        TcpStream,
    },
    OwnedMessage::Text,
    Message,
    message,
    futures::{Future, Stream, Sink},
    //--------------------------------------------------------------//
};
use tokio::runtime::Builder;
use tokio::runtime::Runtime;
use tokio::prelude::Async::{Ready, NotReady};



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

#[derive(Serialize,Debug)]
#[serde(untagged)]
enum GatewayPayloadData {
    Hello(HelloMsg),
    _Empty,
}

// After gateway websocket connection is initiated a hello message is sent from server
#[derive(Deserialize,Serialize,Debug)]
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
            #[derive(Deserialize, Debug)]
            struct Helper {
                op: i32,
                d: serde_json::Value,
                s: i32,
                t: String,
            }

            let helper = Helper::deserialize(deserializer)?;

            fn deserialize_payload_data<'a,T>(val: serde_json::Value) -> Result<T, serde_json::Error> 
                    where for<'de> T: serde::Deserialize<'de>{
                serde_json::from_value(val)            
            }

            let data = match helper.op {
                10 => {
                    match deserialize_payload_data::<HelloMsg>(helper.d) {
                        Ok(m) => Ok(GatewayPayloadData::Hello(m)),
                        Err(e) => Err(e),
                    }
                }
                x => {
                    panic!("Unknown discord gateway payload opcode {}", x);
                },
            };

            match data {
                Ok(data) => {
                    Ok(GatewayPayload{
                        op: helper.op,
                        d: data,
                        s: helper.s,
                        t: helper.t,
                    })
                },
                Err(_) => Err(de::Error::custom("Could not deserialize gateway payload")),
            }
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
// panics if it is unable to connect. Sets version and encoding headers for the
// connection
fn create_websocket(url: &mut Url) -> WsClientSync<WsTlsStreamSync<WsTcpStreamSync>> {
    url.set_query(Some("v=6&encoding=json"));
    let mut ws_client = ClientBuilder::from_url(&url);
    println!("Connecting to {}", url);
    let ws_client = match ws_client.connect_secure(None) {
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
        client: &mut WsClientSync<WsTlsStreamSync<WsTcpStreamSync>>, 
        sleep_ms: u64
        ){
    loop {
        thread_sleep(sleep_ms);
        println!("Loop");

        // Handle heartbeat   
        let resp: message::OwnedMessage = match client.recv_message(){
            Err(e) => {
                println!("Error with receiving message from websocket: {}", e);
                continue;
            },
            Ok(r) => r,
        };
        println!("{:?}",resp);

        
        
    }
}

fn create_websocket_async(url :&mut Url) -> ClientNew<TlsStream<TcpStream>>{
    url.set_query(Some("v=6&encoding=json"));
    // create a Future of a client
    let client_future: ClientNew<TlsStream<TcpStream>> =
        ClientBuilder::from_url(url)
            .async_connect_secure(None);
    client_future
}

fn setup_discord_gateway_async(gateway_url :&mut Url){
    let client_future = create_websocket_async(gateway_url);
    let heartbeat = client_future
        .map(|(mut client, _)| {
            loop {
                let res = client.poll();
                match res {
                    Ok(Ready(x)) => {
                        println!("Received message: {:?}", x);
                        match x {
                            Some(Text(msg)) => {
                                println!("Heartbeat: {:?}", msg);
                            },
                            Some(_) => {println!("Non text heartbeat received")},
                            None => {println!("Found None");},
                        };
                    },
                    Ok(NotReady) => {
                        let t: u64 = 500;
                        println!("No hearbeat. Trying again in {} ms...", t);
                        thread_sleep(t);
                        continue;
                    },
                    Err(_) => {continue;},
                };
            }
        });

    let mut runtime = Builder::new().build().unwrap();
    let res = runtime.block_on(heartbeat);
    println!("Res {:?}", res);
}

pub fn initiate_gateway(client: &HttpClient) -> bool{
    let gateway_url = create_url(&format!("{}gateway/bot", API_BASE_URL));
    let body = send_get(client, &gateway_url);
    let v : GatewayResponse = deserialize(&body);
    let mut gateway_url = create_url(&v.url);

    ///////////////  ASYNC   ////////////////////////////////////
    setup_discord_gateway_async(&mut gateway_url);

    let a = true;
    if a {
        return true;
    }

    //////////////////////////////////////////////////////////////

    //TODO add explicit version and encoding parameters to request

    let mut ws = create_websocket(&mut create_url(&v.url));
    gateway_eventloop_sync(&mut ws, 500);
    
    println!("----\n{:?}\n------", v);
    true
}
