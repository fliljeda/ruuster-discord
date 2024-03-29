use std::env;
use std::fs;
use std::io;

#[derive(Debug)]
enum Flag{
    ConfigFile(String),
}

#[derive(Debug)]
pub struct SettingsInitializer{
    pub client: Option<String>,
    pub guild: Option<String>,
    pub secret: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug)]
pub struct Settings{
    pub client: String,
    pub guild: String,
    pub secret: String,
    pub token: String,
}

impl SettingsInitializer{

    // Convert a settings initializer object to a settings object
    fn finalize(self) -> Settings {
        let mut settings = Settings::new();
        let mapped_vals = vec!{
            (self.client, &mut settings.client),
            (self.guild, &mut settings.guild),
            (self.secret, &mut settings.secret),
            (self.token, &mut settings.token),
        };
        for (map_from, map_to) in mapped_vals {
            *map_to = match map_from {
                None => break,
                Some(s) => s,
            };
        }
        settings
    }
}

impl Settings {
    // Creates empty settings object
    fn new() -> Settings {
        Settings{
            client:String::new(),
            guild:String::new(),
            secret:String::new(),
            token:String::new(),
        }
    }
}

//Extracts command line arguments
fn handle_arguments(settings: &mut SettingsInitializer){
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
fn handle_flags(flags: Vec<Flag>, settings: &mut SettingsInitializer) {
    for f in flags {
        match f {
            Flag::ConfigFile(path) => {
                parse_config_file(&path, settings);
            }
        }
    }
}

// Maps the key to a variable in the settings struct and sets its value to val
fn add_config_option(settings: &mut SettingsInitializer, key: &str, val: &str) {
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

// Parses file from path and sets the configuartions based on the file contents
fn parse_config_file(path: &str, settings: &mut SettingsInitializer){
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

// Outputs prompt to stdout and reads line from stdin and returns as trimmed String
fn prompt_value(prompt: &str) -> String {
    { 
        // print immediately
        use std::io::Write;
        print!("{}", prompt);
        io::stdout().flush().unwrap();
    }

    // read input 
    let mut inp = String::new();
    io::stdin().read_line(&mut inp)
        .expect("Failed to read input");

    // return trimmed string
    String::from(inp.trim())
}

// Checks if certain values in the settings struct is set or not
// Prompts selected values from stdin if not entered
fn handle_missing_configvals(settings: &mut SettingsInitializer) {
    // Vec defining prompts and value handles for when None
    let prompt_handles: Vec<(String,&mut Option<String>)> = vec![
        (String::from("Client id: "), &mut settings.client),
        (String::from("Guild id: "), &mut settings.guild),
        (String::from("Secret: "), &mut settings.secret),
        (String::from("Token: "), &mut settings.token),
    ];
    
    for tup in prompt_handles {
        let prompt = tup.0;
        let val = tup.1;
        *val = match val {
            Some(_) => continue,
            None => Some(prompt_value(&prompt)),
        }
    }

}

// Creates and returns a settings object from where details about runtime specifics
// can be fetched
pub fn get_settings() -> Settings{
    let mut settings = SettingsInitializer{
        client:None,
        guild:None,
        secret:None,
        token:None,
    };

    handle_arguments(&mut settings);
    handle_missing_configvals(&mut settings);
    let settings_final = settings.finalize();
    settings_final
}
