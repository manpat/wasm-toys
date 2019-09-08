use common::*;


// [0, circ]
fn normalise_cyclic(a: f32, circ: f32) -> f32 {
	(a % circ + circ) % circ
}

// [-circ/2, circ/2]
fn diff_cyclic(a: f32, b: f32, circ: f32) -> f32 {
	(a%circ - b%circ + 5.0*circ/2.0) % circ - circ/2.0
}


pub struct ToroidManifold {
	pub circumference: Vec2,
}

impl ToroidManifold {
	pub fn new(circumference: Vec2) -> Self {
		ToroidManifold {circumference}
	}

	pub fn chart(&self, p: Vec2) -> Chart {
		Chart {
			manifold: self,
			proj_point: Vec2{
				x: normalise_cyclic(p.x, self.circumference.x),
				y: normalise_cyclic(p.y, self.circumference.y)
			}
		}
	}

	pub fn difference(&self, a: Vec2, b: Vec2) -> Vec2 {
		Vec2 {
			x: diff_cyclic(a.x, b.x, self.circumference.x),
			y: diff_cyclic(a.y, b.y, self.circumference.y),
		}
	}
}

pub struct Chart<'man> {
	manifold: &'man ToroidManifold,
	proj_point: Vec2,
}

impl<'m> Chart<'m> {
	pub fn to_manifold(&self, p: Vec2) -> Option<Vec2> {
		if p.x.abs() > 1.0 || p.y.abs() > 1.0 {
			return None
		}

		Some(vec2_map!(p, element.asin()/(2.0*PI)) * self.manifold.circumference + self.proj_point)
	}

	pub fn from_manifold(&self, p: Vec2) -> Option<Vec2> {
		let circ = self.manifold.circumference;

		// [-1, 1]
		let x_diff = diff_cyclic(p.x, self.proj_point.x, circ.x) / circ.x * 2.0;
		let y_diff = diff_cyclic(p.y, self.proj_point.y, circ.y) / circ.y * 2.0;

		if x_diff.abs() < 1.0/2.0 && y_diff.abs() < 1.0/2.0 {
			let x_ang = x_diff * PI;
			let y_ang = y_diff * PI;

			Some(Vec2::new(x_ang.sin(), y_ang.sin()))
		} else {
			None
		}
	}
}