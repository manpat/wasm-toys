#![feature(clamp)]

extern crate wasm_toys as engine;
use engine::prelude::*;


fn main() {
	engine::init_engine(App::new);
}


struct App {}

impl App {
	fn new() -> App {
		App {}
	}
}

impl EngineClient for App {
	fn init(&mut self) {
		console_log!("EngineClient init");

		unsafe {
			engine::imports::util::fork(4);
		}

		console_log!("waiting for forks to begin");
	}
	
	fn update(&mut self, _ctx: engine::UpdateContext) {}
}



#[no_mangle]
pub fn worker_main() {
	console_log!("worker_main");
}