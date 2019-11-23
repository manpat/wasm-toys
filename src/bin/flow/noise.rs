use engine::prelude::*;

pub struct Perlin {
	noise: Vec<f32>,
	gradient: Vec<Vec3>,
	size: usize,
}


impl Perlin {
	pub fn new(size: usize) -> Perlin {
		let count = size*size*size;

		let mut noise = vec![0.0; count];
		for g in noise.iter_mut() {
			*g = rand() * 2.0 - 1.0;
		}

		let mut perlin = Perlin {
			noise,
			gradient: vec![Vec3::zero(); count],
			size,
		};

		for z in 0..size {
			for y in 0..size {
				for x in 0..size {
					let cell = Vec3i::new(x as i32, y as i32, z as i32);
					let idx = x + y * size + z * size * size;

					let ax = Vec3i::new(1, 0, 0);
					let ay = Vec3i::new(0, 1, 0);
					let az = Vec3i::new(0, 0, 1);

					let v = perlin.sample_raw(cell);

					let dx = ax.to_vec3() * (perlin.sample_raw(cell + ax) - v);
					let dy = ay.to_vec3() * (perlin.sample_raw(cell + ay) - v);
					let dz = az.to_vec3() * (perlin.sample_raw(cell + az) - v);

					let grad = dx + dy + dz;

					perlin.gradient[idx] = grad;
				}
			}
		}

		perlin
	}

	pub fn sample(&self, p: Vec3) -> f32 {
		let size_f = self.size as f32;

		let wrap = |f| (size_f + (f % size_f)) % size_f;
		let cell_f = vec3_map!(p, wrap(element));
		let cell = vec3i_map!(cell_f, element as i32);
		let uvw = vec3_map!(cell_f, element.fract());

		let a = uvw.x.ease_linear(
			self.sample_raw(cell + Vec3i::new(0, 0, 0)),
			self.sample_raw(cell + Vec3i::new(1, 0, 0))
		);

		let b = uvw.x.ease_linear(
			self.sample_raw(cell + Vec3i::new(0, 1, 0)),
			self.sample_raw(cell + Vec3i::new(1, 1, 0))
		);

		let c = uvw.x.ease_linear(
			self.sample_raw(cell + Vec3i::new(0, 0, 1)),
			self.sample_raw(cell + Vec3i::new(1, 0, 1))
		);

		let d = uvw.x.ease_linear(
			self.sample_raw(cell + Vec3i::new(0, 1, 1)),
			self.sample_raw(cell + Vec3i::new(1, 1, 1))
		);

		let ab = uvw.y.ease_linear(a, b);
		let cd = uvw.y.ease_linear(c, d);

		uvw.z.ease_linear(ab, cd)
	}

	pub fn gradient(&self, p: Vec3) -> Vec3 {
		let size_f = self.size as f32;

		let wrap = |f| (size_f + (f % size_f)) % size_f;
		let cell_f = vec3_map!(p, wrap(element));
		let cell = vec3i_map!(cell_f, element as i32);
		let uvw = vec3_map!(cell_f, element.fract());

		let a = uvw.x.ease_linear(
			self.sample_gradient_raw(cell + Vec3i::new(0, 0, 0)),
			self.sample_gradient_raw(cell + Vec3i::new(1, 0, 0))
		);

		let b = uvw.x.ease_linear(
			self.sample_gradient_raw(cell + Vec3i::new(0, 1, 0)),
			self.sample_gradient_raw(cell + Vec3i::new(1, 1, 0))
		);

		let c = uvw.x.ease_linear(
			self.sample_gradient_raw(cell + Vec3i::new(0, 0, 1)),
			self.sample_gradient_raw(cell + Vec3i::new(1, 0, 1))
		);

		let d = uvw.x.ease_linear(
			self.sample_gradient_raw(cell + Vec3i::new(0, 1, 1)),
			self.sample_gradient_raw(cell + Vec3i::new(1, 1, 1))
		);

		let ab = uvw.y.ease_linear(a, b);
		let cd = uvw.y.ease_linear(c, d);

		uvw.z.ease_linear(ab, cd)
	}

	pub fn sample_raw(&self, p: Vec3i) -> f32 {
		let wrap = |f| if f < 0 {
			self.size as i32 + (f % self.size as i32)
		} else {
			f
		} % self.size as i32;

		let p2 = vec3i_map!(p, wrap(element));

		let idx = p2.x as usize
				+ p2.y as usize * self.size
				+ p2.z as usize * self.size * self.size;

		self.noise[idx as usize]
	}

	pub fn sample_gradient_raw(&self, p: Vec3i) -> Vec3 {
		let wrap = |f| if f < 0 {
			self.size as i32 + (f % self.size as i32)
		} else {
			f
		} % self.size as i32;

		let p2 = vec3i_map!(p, wrap(element));

		let idx = p2.x as usize
				+ p2.y as usize * self.size
				+ p2.z as usize * self.size * self.size;

		self.gradient[idx as usize]
	}
}