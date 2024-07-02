use common::math::*;
use std::cell::Cell;


#[derive(Copy, Clone, Debug)]
pub enum Projection {
	Perspective { fovy: f32 },
	Orthographic { scale: f32 },
}


#[derive(Clone, Debug)]
struct MemoisedMat4 {
	mat: Cell<Mat4>,
	dirty: Cell<bool>
}

impl MemoisedMat4 {
	fn new() -> Self {
		MemoisedMat4 {
			mat: Mat4::identity().into(),
			dirty: Cell::new(true),
		}
	}

	fn mark_dirty(&mut self) { self.dirty.set(true) }

	fn get_or_update<F>(&self, f: F) -> Mat4 where F: FnOnce() -> Mat4 {
		if self.dirty.get() {
			self.mat.set(f());
			self.dirty.set(false);
		}

		self.mat.get()
	}
}


#[derive(Clone, Debug)]
pub struct Camera {
	position: Vec3,
	orientation: Quat,
	projection: Projection,

	near: f32,
	far: f32,
	
	aspect: f32,
	viewport: Vec2i,

	projection_matrix: MemoisedMat4,
	view_matrix: MemoisedMat4,

	inv_projection_matrix: MemoisedMat4,
	inv_view_matrix: MemoisedMat4,

	proj_view_matrix: MemoisedMat4,
	inv_proj_view_matrix: MemoisedMat4,
}


impl Camera {
	pub fn new() -> Self {
		Camera {
			position: Vec3::zero(),
			orientation: Quat::identity(),
			projection: Projection::Perspective{ fovy: PI/3.0 },

			near: 0.1, far: 100.0,

			aspect: 1.0,
			viewport: Vec2i::splat(1),

			projection_matrix: MemoisedMat4::new(),
			view_matrix: MemoisedMat4::new(),
			proj_view_matrix: MemoisedMat4::new(),
			
			inv_projection_matrix: MemoisedMat4::new(),
			inv_view_matrix: MemoisedMat4::new(),
			inv_proj_view_matrix: MemoisedMat4::new(),
		}
	}

	pub fn position(&self) -> Vec3 { self.position }
	pub fn orientation(&self) -> Quat { self.orientation }
	pub fn viewport(&self) -> Vec2i { self.viewport }
	pub fn aspect(&self) -> f32 { self.aspect }

	pub fn update(&mut self, viewport: Vec2i) {
		self.viewport = viewport;
		let viewport = viewport.to_vec2();
		let aspect = viewport.x / viewport.y;

		if (self.aspect - aspect).abs() > 0.0 {
			self.aspect = aspect;
			self.mark_projection_dirty();
		}
	}

	pub fn projection_matrix(&self) -> Mat4 {
		self.projection_matrix.get_or_update(|| {
			use self::Projection::*;

			match self.projection {
				Perspective{fovy} => Mat4::perspective(fovy, self.aspect, self.near, self.far),
				Orthographic{scale} => Mat4::ortho_aspect(scale, self.aspect, self.near, self.far)
			}
		})
	}


	pub fn view_matrix(&self) -> Mat4 {
		self.view_matrix.get_or_update(|| {
			self.orientation.conjugate().to_mat4() * Mat4::translate(-self.position)
		})
	}


	pub fn projection_view(&self) -> Mat4 {
		self.proj_view_matrix.get_or_update(|| {
			 self.projection_matrix() * self.view_matrix()
		})
	}


	pub fn inverse_projection_matrix(&self) -> Mat4 {
		self.inv_projection_matrix.get_or_update(|| {
			self.projection_matrix().inverse()
		})
	}

	pub fn inverse_view_matrix(&self) -> Mat4 {
		self.inv_view_matrix.get_or_update(|| {
			self.view_matrix().inverse()
		})
	}

	pub fn inverse_projection_view(&self) -> Mat4 {
		self.inv_proj_view_matrix.get_or_update(|| {
			 self.projection_view().inverse()
		})
	}


	fn mark_projection_dirty(&mut self) {
		self.projection_matrix.mark_dirty();
		self.proj_view_matrix.mark_dirty();
	}

	fn mark_view_dirty(&mut self) {
		self.view_matrix.mark_dirty();
		self.proj_view_matrix.mark_dirty();
	}


	pub fn set_projection(&mut self, p: Projection) {
		self.projection = p;
		self.mark_projection_dirty();
	}

	pub fn set_near_far(&mut self, n: f32, f: f32) {
		self.near = n;
		self.far = f;
		self.mark_projection_dirty();
	}


	pub fn set_position(&mut self, p: Vec3) {
		self.position = p;
		self.mark_view_dirty();
	}

	pub fn set_orientation(&mut self, q: Quat) {
		self.orientation = q;
		self.mark_view_dirty();
	}

	pub fn set_euler(&mut self, angles: Vec3) {
		let yaw = Quat::new(Vec3::from_y(1.0), angles.y);
		let roll = Quat::new(Vec3::from_z(1.0), angles.z);
		let pitch = Quat::new(Vec3::from_x(1.0), angles.x);

		self.orientation = (roll * pitch * yaw).normalize();
		self.mark_view_dirty();
	}



	pub fn screen_to_world(&self, screen: Vec3) -> Vec3 {
		let v = self.inverse_projection_view() * screen.extend(1.0);
		v.to_vec3() / v.w
	}
}

