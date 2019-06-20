mod config;
mod discord;
extern crate reqwest;

fn main() {
    let settings = config::get_settings();
    match discord::test_connection(&settings) {
        Ok(_) => println!("Connection OK"),
        Err(s) => {
            println!("Error with connectiontest: {}", s);
            println!("Closing application");
            return;
        },
    }
    discord::start_bot(&settings);
}
