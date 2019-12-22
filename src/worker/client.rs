

pub struct Client {
	client: Box<dyn WorkerClient>,
}


impl Client {
	pub fn new<C: WorkerClient + 'static>(client: C) -> Self {
		Client {
			client: box client,
		}
	}
}


pub trait WorkerClient {
	fn init(&mut self) {}
}

