use std::env;
use std::fs;

#[derive(Debug)]
enum Flag{
    ConfigFile(String),
}

#[derive(Debug)]
pub struct Settings{
    client: Option<String>,
    guild: Option<String>,
    secret: Option<String>,
    token: Option<String>,
}

//Extracts command line arguments
fn handle_arguments(settings: &mut Settings){
    let mut flags: Vec<Flag> = Vec::new();
    let mut args_iter = env::args();
    while let Some(arg) = args_iter.next() {
        if arg == "-f" {
            let file_path = match args_iter.next() {
                Some(p) => {p},
                None => {panic!("Config file path must follow flag -f")}
            };
   
            flags.push(Flag::ConfigFile(file_path));
        }
    }
    handle_flags(flags, settings);
    
}

// Decides what to do with the given flags
fn handle_flags(flags: Vec<Flag>, settings: &mut Settings) {
    for f in flags {
        match f {
            Flag::ConfigFile(path) => {
                parse_config_file(&path, settings);
            }
        }
    }
}

fn add_config_option(settings: &mut Settings, key: &str, val: &str) {
    match key {
        "client" => {
            settings.client = Some(String::from(val.clone()));
        },
        "guild" =>  {
            settings.guild = Some(String::from(val.clone()));
        },

        "secret" =>  {
            settings.secret = Some(String::from(val.clone()));
        },

        "token" =>  {
            settings.token = Some(String::from(val.clone()));
        },

        &_ => {return;}
    };
}

fn parse_config_file(path: &str, settings: &mut Settings){
    let contents = fs::read_to_string(path)
        .expect("Could not read the file");
    let lines = contents.lines();
    for line in lines {
        let line = line.trim();
        let values: Vec<&str> = line.split('=').collect();
        if values.len() != 2 {
            panic!(String::from("config file contains bad line: ") + line);
        }

        add_config_option(settings, values[0], values[1]);
    }
}


// Creates and returns a settings object from where details about runtime specifics
// can be fetched
pub fn get_settings() -> Settings {
    let mut settings = Settings{
        client:None,
        guild:None,
        secret:None,
        token:None,
    };

    handle_arguments(&mut settings);
    settings
}
