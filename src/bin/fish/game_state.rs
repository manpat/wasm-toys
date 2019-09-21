use engine::prelude::*;


#[derive(Debug, Clone, Hash, PartialEq)]
pub enum Item {
	Bucket{ filled: bool },
	Fish{ scaled: bool },
	Soup(Vec<Item>),
}


#[derive(Debug, Hash)]
pub struct GameState {
	pub soup: SoupState,
	pub bench: BenchState,
	pub fishing_hole: FishingHoleState,
	pub shelf: ShelfState,

	pub inventory: Option<Item>,
}


#[derive(Debug, Hash)]
pub struct SoupState {
	pub inventory: Vec<Item>,
}


#[derive(Debug, Hash)]
pub struct BenchState {
	pub inventory: Option<Item>,
}


#[derive(Debug, Hash)]
pub struct FishingHoleState {
	pub fish: bool,
}

#[derive(Debug, Hash)]
pub struct ShelfState {
	pub bucket: bool,
}



impl GameState {
	pub fn new() -> Self {
		GameState {
			soup: SoupState {
				inventory: Vec::new(),
			},

			bench: BenchState {
				inventory: None,
			},

			fishing_hole: FishingHoleState {
				fish: true,
			},

			shelf: ShelfState {
				bucket: true,
			},

			inventory: None,
		}
	}

	pub fn interact(&mut self, id: &str) {
		console_log!("INTERACTION {}!", id);

		match id {
			"IT_Cauldron" => self.soup.interact(&mut self.inventory),
			"IT_Bench" => self.bench.interact(&mut self.inventory),
			"IT_Shelf" => self.shelf.interact(&mut self.inventory),
			"IT_WaterHole" => if let Some(Item::Bucket{ ref mut filled }) = self.inventory {
				console_log!("Filled bucket!");
				*filled = true;
			}

			"IT_FishingHole" => if self.inventory.is_none() && self.fishing_hole.fish {
				self.fishing_hole.fish = false;
				self.inventory = Some(Item::Fish{ scaled: false });
			}

			_ => {}
		}
	}
}



impl SoupState {
	fn interact(&mut self, ply_inv: &mut Option<Item>) {
		// try place broth first, and give bucket back
		if !self.contains_broth() {
			if ply_inv == &Some(Item::Bucket{ filled: true }) {
				self.inventory.push(ply_inv.take().unwrap());
				*ply_inv = Some(Item::Bucket{ filled: false });
			}
			return;
		}

		// place from player inventory
		if ply_inv.is_some() && self.can_place(ply_inv.as_ref().unwrap()) {
			self.inventory.push(ply_inv.take().unwrap());
			return;
		}

		// take from bench
		if self.is_valid_soup() && ply_inv.is_none() {
			*ply_inv = Some(self.take_soup());
		}
	}

	pub fn is_valid_soup(&self) -> bool { self.inventory.len() > 1 }

	fn take_soup(&mut self) -> Item {
		let ingredients = std::mem::replace(&mut self.inventory, Vec::new());
		console_log!("Making soup with {:?}", ingredients);
		Item::Soup(ingredients)
	}

	fn contains_broth(&self) -> bool {
		self.inventory.contains(&Item::Bucket{ filled: true })
	}

	fn can_place(&self, item: &Item) -> bool {
		match item {
			Item::Fish{ scaled: true } => true,
			_ => false,
		}
	}
}

impl BenchState {
	fn interact(&mut self, ply_inv: &mut Option<Item>) {
		// place from player inventory
		if ply_inv.is_some() && self.can_place(ply_inv.as_ref().unwrap()) {
			self.inventory = ply_inv.take();
			return;
		}

		// interact with thing in inventory
		if self.inventory_interact() {
			return;
		}

		// take from bench
		if self.inventory.is_some() && ply_inv.is_none() {
			*ply_inv = self.inventory.take();
		}
	}

	fn can_place(&self, item: &Item) -> bool {
		if self.inventory.is_some() {
			return false
		}

		match item {
			Item::Fish{ scaled: false } => true,
			_ => false,
		}
	}

	fn inventory_interact(&mut self) -> bool {
		match self.inventory {
			Some(Item::Fish{ scaled: false }) => {
				self.inventory = Some(Item::Fish{ scaled: true });
				true
			}

			_ => false
		}
	}
}

impl ShelfState {
	fn interact(&mut self, ply_inv: &mut Option<Item>) {
		// place from player inventory
		if ply_inv.is_some() && self.can_place(ply_inv.as_ref().unwrap()) {
			self.bucket = true;
			*ply_inv = None;
			return;
		}

		// take from bench
		if self.bucket && ply_inv.is_none() {
			*ply_inv = Some(Item::Bucket{ filled: false });
			self.bucket = false;
		}
	}

	fn can_place(&self, item: &Item) -> bool {
		match item {
			Item::Bucket{ filled: false } => !self.bucket,
			_ => false,
		}
	}
}

impl FishingHoleState {
	
}