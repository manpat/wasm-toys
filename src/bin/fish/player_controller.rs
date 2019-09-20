use engine::prelude::*;

pub struct PlayerController {
	pub pos: Vec3,
	pub rot: Quat,

	yaw: f32,
	yaw_vel: f32,

	drag_vel: f32,
}

impl PlayerController {
	pub fn new() -> Self {
		PlayerController {
			pos: Vec3::zero(),
			rot: Quat::ident(),

			yaw: 0.0,
			yaw_vel: 0.0,

			drag_vel: 0.0,
		}
	}

	pub fn update(&mut self, ctx: &engine::UpdateContext, aspect: f32) {
		// spin
		if ctx.input.dragging() {
			let raw_delta = ctx.input.frame_delta();
			let delta = -raw_delta.x as f32 * PI * aspect;
			self.yaw_vel += (delta - self.yaw_vel) / 3.0;

			let drag_delta = ctx.input.drag_delta();
			self.pos += self.rot.forward() * DT * drag_delta.y * 12.0;

		} else {
			self.yaw_vel *= 0.5;
		}

		self.yaw += self.yaw_vel;


		self.rot = Quat::new(Vec3::from_y(1.0), self.yaw);

		// movement
		use engine::input::*;

		if ctx.input_raw.button_state(KeyCode::W.into()).is_down() {
			self.pos += self.rot.forward() * 6.0 * DT;
		}

		if ctx.input_raw.button_state(KeyCode::S.into()).is_down() {
			self.pos -= self.rot.forward() * 6.0 * DT;
		}

		if ctx.input_raw.button_state(KeyCode::D.into()).is_down() {
			self.pos += self.rot.right() * 6.0 * DT;
		}

		if ctx.input_raw.button_state(KeyCode::A.into()).is_down() {
			self.pos -= self.rot.right() * 6.0 * DT;
		}
	}

	pub fn update_camera(&self, camera: &mut Camera) {
		camera.set_orientation(self.rot);
		camera.set_position(self.pos + Vec3::from_y(1.4));
	}
}

