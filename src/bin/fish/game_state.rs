use engine::prelude::*;


#[derive(Debug, Copy, Clone, PartialEq, Hash)]
pub struct GameState {
	pub soup_exists: bool
}


impl GameState {
	pub fn new() -> Self {
		GameState {
			soup_exists: false
		}
	}

	pub fn interact(&mut self, id: &str) {
		console_log!("INTERACTION {}!", id);

		match id {
			"IT_Cauldron" => {
				self.soup_exists = !self.soup_exists;
			}

			_ => {}
		}
	}
}