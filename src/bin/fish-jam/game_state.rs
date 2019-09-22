use engine::prelude::*;


#[derive(Debug, Clone, Hash, PartialEq)]
pub enum Item {
	Bucket{ filled: bool },
	Fish{ variant: String },
	Soup(Vec<Item>),
	EmptyBowl,
	// Coin,
}


#[derive(Debug, Hash)]
pub struct GameState {
	pub cauldron: CauldronState,
	pub bench: BenchState,
	pub fishing_hole: FishingHoleState,
	pub shelf: ShelfState,
	pub table: TableState,

	pub inventory: Option<Item>,

	pub in_bed: bool,
}


#[derive(Debug, Hash)]
pub struct CauldronState {
	pub inventory: Vec<Item>,
}


#[derive(Debug, Hash)]
pub struct BenchState {
	pub inventory: Option<Item>,
}


#[derive(Debug, Hash)]
pub struct FishingHoleState {
	pub red_fish: bool,
	pub blue_fish: bool,
	pub green_fish: bool,
	pub orange_fish: bool,
}

#[derive(Debug, Hash)]
pub struct ShelfState {
	pub inventory: Option<Item>,
}


#[derive(Debug, Hash)]
pub struct TableState {
	pub inventory: Vec<Item>,
}



impl GameState {
	pub fn new() -> Self {
		GameState {
			cauldron: CauldronState {
				inventory: Vec::new(),
			},

			bench: BenchState {
				inventory: None,
			},

			fishing_hole: FishingHoleState {
				red_fish: true,
				blue_fish: true,
				green_fish: true,
				orange_fish: true,
			},

			shelf: ShelfState {
				inventory: Some(Item::Bucket{ filled: false }),
			},

			table: TableState {
				inventory: Vec::new(),
			},

			inventory: None,

			in_bed: false,
		}
	}

	pub fn interact(&mut self, id: &str) {
		match id {
			"IT_Cauldron" => self.cauldron.interact(&mut self.inventory),
			"IT_Bench" => self.bench.interact(&mut self.inventory),
			"IT_Shelf" => self.shelf.interact(&mut self.inventory),
			"IT_Table" => self.table.interact(&mut self.inventory),

			"IT_WaterHole" => if let Some(Item::Bucket{ ref mut filled }) = self.inventory {
				*filled = true;
			}

			"IT_FishinHole" => self.fishing_hole.interact(&mut self.inventory),

			"IT_Bed" => { self.in_bed = true }

			_ => panic!("Unknown interaction! {}", id)
		}
	}
}



impl CauldronState {
	fn interact(&mut self, ply_inv: &mut Option<Item>) {
		// take empty bucket
		if self.contains_bucket() {
			if ply_inv.is_none() {
				*ply_inv = Some(Item::Bucket{ filled: false });
				self.inventory.clear();
			}
			return;
		}

		// try place broth first, and give bucket back
		if !self.contains_broth() {
			if ply_inv == &Some(Item::Bucket{ filled: true }) {
				self.inventory.push(ply_inv.take().unwrap());
				*ply_inv = Some(Item::Bucket{ filled: false });

			} else if ply_inv == &Some(Item::Bucket{ filled: false }) {
				self.inventory.push(ply_inv.take().unwrap());
				*ply_inv = None;

			} else if let Some(Item::Soup(ingredients)) = ply_inv {
				self.inventory = std::mem::replace(ingredients, Vec::new());
				*ply_inv = None;
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

	fn contains_bucket(&self) -> bool {
		self.inventory.contains(&Item::Bucket{ filled: false })
	}

	fn can_place(&self, item: &Item) -> bool {
		match item {
			Item::Fish{ variant } => variant == "scaled",
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
			Item::Fish{ .. } => true,
			_ => false,
		}
	}

	fn inventory_interact(&mut self) -> bool {
		match &self.inventory {
			Some(Item::Fish{ variant }) if (variant != "scaled") => {
				self.inventory = Some(Item::Fish{ variant: "scaled".into() });
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
			self.inventory = ply_inv.take();
			return;
		}

		// take from bench
		if self.inventory.is_some() && ply_inv.is_none() {
			*ply_inv = self.inventory.take();
		}
	}

	fn can_place(&self, item: &Item) -> bool {
		match item {
			Item::Bucket{ .. } => self.inventory.is_none(),
			_ => false,
		}
	}
}

impl TableState {
	fn interact(&mut self, ply_inv: &mut Option<Item>) {
		// place from player inventory
		if ply_inv.is_some() && self.can_place(ply_inv.as_ref().unwrap()) {
			self.inventory.push(ply_inv.take().unwrap());
			return;
		}

		// eat soup
		if !self.inventory.is_empty() {
			let bowl = self.inventory.iter_mut().last().unwrap();
			*bowl = Item::EmptyBowl;
		}
	}

	fn can_place(&self, item: &Item) -> bool {
		match item {
			Item::Soup(_) => self.inventory.is_empty() || self.inventory.iter().last() == Some(&Item::EmptyBowl),
			_ => false,
		}
	}
}

impl FishingHoleState {
	fn interact(&mut self, ply_inv: &mut Option<Item>) {
		let mut variants = vec![
			(&mut self.red_fish, "red"),
			(&mut self.green_fish, "green"),
			(&mut self.orange_fish, "orange"),
			(&mut self.blue_fish, "blue")
		];

		// try put fish back
		if let Some(Item::Fish{ variant }) = ply_inv.as_ref() {
			if let Some(v) = variants.iter_mut().find(|(_, n)| n == variant) {
				*v.0 = true;
				*ply_inv = None; 
			}
			return;
		}

		// don't overwrite player inventory
		if ply_inv.is_some() { return }

		// try take fish
		variants.retain(|(f, _)| **f);
		if variants.is_empty() { return }

		let num_valid_variants = variants.len();

		let (ref mut fish, variant) = variants[(rand() * 100.0).floor() as usize % num_valid_variants];

		*ply_inv = Some(Item::Fish{ variant: variant.into() });
		**fish = false;
	}
}