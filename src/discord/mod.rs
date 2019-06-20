use super::config::Settings;
use reqwest::{Client, Url};

pub fn test_connection(settings: &Settings) -> Result<(), reqwest::Error>{
    println!("Client: {:?}", settings.client.as_ref());
    reqwest::get("https://httpbin.org/get")?;
    Ok(())
}

// TODO: Tests config values' validity for the discord API
fn settings_valid(settings: &Settings) -> bool {
    match settings.client {
        Some(_) => true,
        None => false,
    }
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
    auth.push_str(settings.token.as_ref().unwrap());
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

// Starts the bot using Bot Token Authorization Header
// https://discordapp.com/developers/docs/reference#authentication
pub fn start_bot(settings: &Settings) {
    assert!(settings_valid(settings));
    println!("{}", settings.client.as_ref().unwrap());
    let client = match build_client(&settings) {
        Ok(c) => c,
        Err(e) => panic!(e),
    };


    //TODO fix error handling
    // TODO refactor to functions
    let api_base_url = "https://discordapp.com/api/";
    let guild = settings.guild.as_ref().unwrap();
    let url = Url::parse(&format!("{}guilds/{}/channels", 
                                 api_base_url, 
                                 guild)).unwrap();

    println!("{:?}", url);
    let mut resp = client.get(url).send().unwrap();
    let body = resp.text().unwrap();
    println!("{}", body);


}
