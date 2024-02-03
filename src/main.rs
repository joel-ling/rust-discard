#![feature(coroutine_trait)]
#![feature(coroutines)]

mod aliases;
mod discard_server;

use crate::discard_server::DiscardServer;

fn main() {
    const ADDRESS: &str = "localhost:5009";

    match DiscardServer::new(ADDRESS) {
        Ok(()) => {}

        Err(error) => panic!("{error}"),
    }
}
