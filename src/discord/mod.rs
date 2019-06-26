use super::config::Settings;
use reqwest::{Client, Url};
use serde::{Deserialize};
use std::fmt;
use std::{thread, time};

const API_BASE_URL: &str = "https://discordapp.com/api/";

#[derive(PartialEq)]
enum ChannelType {
    Text = 0,
    DM = 1,
    Voice = 2,
    GroupDM = 3,
    GuildCategory = 4,
    GuildNews = 5,
    GuildStore = 6,
    Unknown,
}

// Channel fields directly corresponds to a subset of the Discord api
// channel response json
#[derive(Deserialize, Debug)]
struct Channel{
    #[serde(alias = "type")]
    ctype: u8,
    id: String,
    name: String,
    last_message_id: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Message{
    id: String,
    channel_id: String,
    content: String,
}

impl ChannelType {
    fn from_u8(u :u8) -> ChannelType {
        match u {
            0 => ChannelType::Text,
            1 => ChannelType::DM,
            2 => ChannelType::Voice,
            3 => ChannelType::GroupDM,
            4 => ChannelType::GuildCategory,
            5 => ChannelType::GuildNews,
            6 => ChannelType::GuildStore,
            _ => ChannelType::Unknown,
        }
    }
}

impl fmt::Display for ChannelType {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            ChannelType::Text => "Text",
            ChannelType::Voice => "Voice",
            ChannelType::Unknown => "Unknown",
            _ => "Not text/voice",
        };
        write!(f, "{}", text)
    }
}

impl Channel {
    fn get_channel_type(&self) -> ChannelType {
        ChannelType::from_u8(self.ctype)
    }

    fn update_last_message(&mut self, msg: &Message){
        self.last_message_id = Some(msg.id.clone());
    }

    fn get_new_messages(&mut self, client: &Client) -> Vec<Message> {
        let url = Url::parse(&format!("{}channels/{}/messages", 
                                     API_BASE_URL, 
                                     self.id)).unwrap();
        let mut req = client.get(url.clone());

        req = req.query(&[("after", self.last_message_id.as_ref().unwrap())]);
                                
        let mut resp = req.send().unwrap();
        let body = resp.text().unwrap();
        let v : Vec<Message> = match serde_json::from_str(&body) {
            Err(e) => {
                println!("Something went wrong with deserializing json from url: {}", url);
                panic!(e);
            },
            Ok(a) => a,
        };

        // Update the channel's last sent message id if new messages are found
        // Returned messages starts from last sent message
        if let Some(msg) = v.first().as_ref() {
            self.update_last_message(msg);
        }

        v
    }
}
impl fmt::Display for Channel {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Channel(type=\"{}\" name=\"{}\")", self.get_channel_type(), self.name)
    }
}

pub fn test_connection(settings: &Settings) -> Result<(), reqwest::Error>{
    println!("Client: {:?}", settings.client);
    reqwest::get("https://httpbin.org/get")?;
    Ok(())
}


// TODO: Tests config values' validity for the discord API
fn settings_valid(_settings: &Settings) -> bool {
    true
}

// Creates a headervalue struct from a string value
fn get_as_header(val: &str) -> reqwest::header::HeaderValue {
    //TODO fix error handling
    let auth_val = reqwest::header::HeaderValue::from_str(val)
        .expect(format!("Could not create header value from: {}", val).as_ref());
    auth_val
}

fn gen_default_headers(settings: &Settings) -> reqwest::header::HeaderMap {
    use reqwest::header;
    let mut headers = header::HeaderMap::new();

    // TODO refactor to functions
    let mut auth = String::from("Bot ");
    auth.push_str(settings.token.as_ref());
    let auth_val = get_as_header(&auth);

    headers.insert(header::AUTHORIZATION, auth_val);
    headers
}

// Builds and sets the default values for a http client
fn build_client(settings: &Settings) -> Result<Client, reqwest::Error> {
    let headers = gen_default_headers(settings);
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .default_headers(headers)
        .build()?;

    Ok(client)
}


fn get_channels(client: &Client, guild: &str) -> Vec<Channel> {
    let url = Url::parse(&format!("{}guilds/{}/channels", 
                                 API_BASE_URL, 
                                 guild)).unwrap();

    let mut resp = client.get(url.clone()).send().unwrap();
    let body = resp.text().unwrap();
    let v : Vec<Channel> = match serde_json::from_str(&body) {
        Err(e) => {
            println!("Something went wrong with deserializing json from url: {}", url);
            panic!(e);
        },
        Ok(a) => a,
    };
    v
}

fn get_text_channels(client: &Client, guild: &str) -> Vec<Channel> {
    let mut v = get_channels(client, guild);
    v.retain(|c| c.get_channel_type() == ChannelType::Text);
    v
}




// Starts the bot using Bot Token Authorization Header
// https://discordapp.com/developers/docs/reference#authentication
pub fn start_bot(settings: &Settings) {
    assert!(settings_valid(settings));
    println!("{}", &settings.client);
    let client = match build_client(&settings) {
        Ok(c) => c,
        Err(e) => panic!(e),
    };
    let mut v = get_text_channels(&client, &settings.guild);

    loop {

        for x in &mut v {
            let msgs = x.get_new_messages(&client);
            for msg in msgs.iter().rev() {
                println!("New message: {:?}", msg);
            }
        }

        let ten_millis = time::Duration::from_millis(3000);
        println!("Sleeping 3000 ms");
        thread::sleep(ten_millis);
    }
}
