use common::math::Vec2i;

#[derive(Clone, Debug)]
pub struct Map<T> {
	data: Vec<T>,
	size: Vec2i,
}

impl<T> Map<T> where T: Clone {
	pub fn new(size: Vec2i, default: T) -> Map<T> {
		Map {
			data: vec![default; (size.x*size.y) as usize],
			size,
		}
	}

	pub fn new_empty() -> Map<T> {
		Map {
			data: Vec::new(), size: Vec2i::zero(),
		}
	}

	pub fn fill_with(&mut self, val: T) {
		for t in self.data.iter_mut() {
			*t = val.clone();
		}
	}

	pub fn in_bounds(&self, p: Vec2i) -> bool {
		p.x >= 0 && p.x < self.size.x
		&& p.y >= 0 && p.y < self.size.y
	}

	pub fn is_empty(&self) -> bool { self.data.is_empty() }

	pub fn size(&self) -> Vec2i { self.size }

	pub fn get(&self, p: Vec2i) -> Option<&T> {
		if self.in_bounds(p) {
			let idx = (p.x + p.y * self.size.x) as usize;
			self.data.get(idx)
		} else {
			None
		}
	}

	pub fn get_mut(&mut self, p: Vec2i) -> Option<&mut T> {
		if self.in_bounds(p) {
			let idx = (p.x + p.y * self.size.x) as usize;
			self.data.get_mut(idx)
		} else {
			None
		}
	}

	pub fn iter(&self) -> impl Iterator<Item=(&T, Vec2i)> {
		let sx = self.size.x;

		self.data.iter().enumerate()
			.map(move |(i, d)| {
				let i = i as i32;
				let c = Vec2i::new(i%sx, i/sx);
				(d, c)
			})
	}
}