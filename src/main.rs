use neur::Config;

fn main() {
    let config = Config::parse().unwrap();
    dbg!(config);
}
