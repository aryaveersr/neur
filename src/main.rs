use neur::{Config, Generator};

fn main() {
    let config = Config::parse().unwrap();

    dbg!(&config);
    Generator::new(config).unwrap().run().unwrap();
}
