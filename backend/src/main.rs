use std::thread;

use backend::api::GomokuClient;
use websocket::sync::Server;

fn main() {
	let port = 8000u16;

	let server = Server::bind(format!("localhost:{}", port))
		.unwrap_or_else(|_| panic!("Failed to bind server to port {}", port));

	for request in server.filter_map(Result::ok) {
		thread::spawn(|| match GomokuClient::new(request) {
			Err(e) => println!("Failed to start new client: {}", e),
			Ok(mut c) => c.handle(),
		});
	}
}
