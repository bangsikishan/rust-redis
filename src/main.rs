mod redis;
mod server;

use server::Server;

fn main() {
    let mut server = Server::new();
    server.listen();
}
