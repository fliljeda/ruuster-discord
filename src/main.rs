mod config;
fn main() {
    let settings = config::get_settings();
    println!("{:?}", settings);
}
