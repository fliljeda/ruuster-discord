use super::config::Settings;
use reqwest::{Client, Url};
use serde::{Deserialize};
use std::fmt;

const API_BASE_URL: &str = "https://discordapp.com/api/";

#[derive(Deserialize)]
enum ChannelType {
    Text = 0,
    _DM = 1,
    Voice = 2,
    _GroupDM = 3,
    _GuildCategory = 4,
    _GuildNews = 5,
    _GuildStore = 6,
}

#[derive(Deserialize)]
struct Channel{
    #[serde(alias = "type")]
    ctype: ChannelType,
    name: String,
}


impl fmt::Display for ChannelType {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            ChannelType::Text => "Text",
            ChannelType::Voice => "Voice",
            _ => "Not text/voice",
        };
        write!(f, "{}", text)
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Channel(type=\"{}\" name=\"{}\")", self.ctype, self.name)
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

    println!("{:?}", url);
    let mut resp = client.get(url).send().unwrap();
    let body = resp.text().unwrap();
    //println!("{}", body);
    let v : Vec<Channel> = match serde_json::from_str(&body) {
        Err(e) => panic!(e),
        Ok(a) => a,
    };
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
    get_channels(&client, &settings.guild);
}
