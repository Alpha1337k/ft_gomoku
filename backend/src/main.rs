use backend::api::GomokuServer;

fn main() {
	let server = GomokuServer {
		port: 8000u16
	};

	server.start().expect("Server failed to start: ")
}