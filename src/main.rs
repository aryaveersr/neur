use neur::{Config, Generator};

fn main() {
    let config = Config::parse().unwrap();
    Generator::new(config).run().unwrap();
}
