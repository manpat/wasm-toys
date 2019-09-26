use engine::prelude::*;

const MAX_PLAYER_DIST: f32 = 92.0;
const PLAYER_SPEED: f32 = 6.0;
const MAX_PITCH: f32 = PI / 3.0;

pub struct PlayerController {
	pub pos: Vec3,
	pub rot: Quat,

	yaw: f32,
	yaw_vel: f32,

	pitch: f32,
	pitch_vel: f32,
}

impl PlayerController {
	pub fn new() -> Self {
		PlayerController {
			pos: Vec3::zero(),
			rot: Quat::ident(),

			yaw: 0.0,
			yaw_vel: 0.0,

			pitch: 0.0,
			pitch_vel: 0.0,
		}
	}

	pub fn update(&mut self, ctx: &engine::UpdateContext, aspect: f32) {
		// spin
		if ctx.input.dragging() {
			let raw_delta = ctx.input.frame_delta();
			let delta = -raw_delta.x as f32 * PI * aspect;
			self.yaw_vel += (delta - self.yaw_vel) / 3.0;

			let drag_delta = ctx.input.drag_delta();
			let drag_move_thresh = 10.0 / ctx.viewport.y as f32;
			if drag_delta.y.abs() > drag_move_thresh {
				let drag_move_offset = drag_move_thresh.copysign(drag_delta.y);
				let delta = (drag_delta.y + drag_move_offset) * PLAYER_SPEED * 2.0;

				self.pos += self.rot.forward() * DT * delta.clamp(-PLAYER_SPEED, PLAYER_SPEED);
			}

		} else if ctx.input_raw.is_pointer_locked() {
			let raw_delta = ctx.input.frame_delta();
			let yaw_delta = -raw_delta.x as f32 * PI * aspect;
			let pitch_delta = raw_delta.y as f32 * PI * aspect;
			self.yaw_vel += (yaw_delta / 2.0 - self.yaw_vel) / 5.0;
			self.pitch_vel += (pitch_delta / 4.0 - self.pitch_vel) / 3.0;

		} else {
			self.yaw_vel *= 0.5;
			self.pitch_vel *= 0.5;
		}

		self.pitch = (self.pitch + self.pitch_vel).clamp(-MAX_PITCH, MAX_PITCH);
		self.yaw += self.yaw_vel;


		self.rot = Quat::new(Vec3::from_y(1.0), self.yaw);
		// movement
		use engine::input::*;

		if ctx.input_raw.button_state(KeyCode::W).is_down() {
			self.pos += self.rot.forward() * PLAYER_SPEED * DT;
		}

		if ctx.input_raw.button_state(KeyCode::S).is_down() {
			self.pos -= self.rot.forward() * PLAYER_SPEED * DT;
		}

		if ctx.input_raw.button_state(KeyCode::D).is_down() {
			self.pos += self.rot.right() * PLAYER_SPEED * DT;
		}

		if ctx.input_raw.button_state(KeyCode::A).is_down() {
			self.pos -= self.rot.right() * PLAYER_SPEED * DT;
		}

		// keep near the center
		let player_dist = self.pos.to_xz().length();
		if player_dist > MAX_PLAYER_DIST {
			let to_center = -self.pos.to_xz() / player_dist;

			let amt = (player_dist - MAX_PLAYER_DIST).powi(2);
			self.pos += (to_center * amt).to_x0z() * DT;
		}
	}

	pub fn update_camera(&self, camera: &mut Camera) {
		camera.set_orientation(self.rot * Quat::new(Vec3::from_x(1.0), self.pitch));
		camera.set_position(self.pos + Vec3::from_y(1.4));
	}
}

