use reqwest::{Client, Url};
use super::API_BASE_URL;
use serde::{Deserialize};
//use std::fmt;


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
    pub d: T, // Data (unspecified format)
    pub s: i32, // Sequence number
    pub t: String,  // Event name
}

fn send_get(client: &Client, url: &str) -> String {
    let url = Url::parse(url).unwrap();
    let req = client.get(url.clone());
                            
    let mut resp = req.send().unwrap();
    let body = resp.text().unwrap();
    body
}

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

pub fn initiate_gateway(client: &Client) -> bool{
    let url = &format!("{}gateway/bot", API_BASE_URL);
    let body = send_get(client, url);
    let v : GatewayResponse = deserialize(&body);
    println!("----\n{:?}\n------", v);
    true
}
