use engine::prelude::*;

const MAX_PLAYER_DIST: f32 = 92.0;
const PLAYER_SPEED: f32 = 6.0;

pub struct PlayerController {
	pub pos: Vec3,
	pub rot: Quat,

	yaw: f32,
	yaw_vel: f32,
}

impl PlayerController {
	pub fn new() -> Self {
		PlayerController {
			pos: Vec3::zero(),
			rot: Quat::ident(),

			yaw: 0.0,
			yaw_vel: 0.0,
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
			let delta = -raw_delta.x as f32 * PI * aspect;
			self.yaw_vel += (delta / 2.0 - self.yaw_vel) / 5.0;

		} else {
			self.yaw_vel *= 0.5;
		}

		self.yaw += self.yaw_vel;


		self.rot = Quat::new(Vec3::from_y(1.0), self.yaw);

		// movement
		use engine::input::*;

		if ctx.input_raw.button_state(KeyCode::W.into()).is_down() {
			self.pos += self.rot.forward() * PLAYER_SPEED * DT;
		}

		if ctx.input_raw.button_state(KeyCode::S.into()).is_down() {
			self.pos -= self.rot.forward() * PLAYER_SPEED * DT;
		}

		if ctx.input_raw.button_state(KeyCode::D.into()).is_down() {
			self.pos += self.rot.right() * PLAYER_SPEED * DT;
		}

		if ctx.input_raw.button_state(KeyCode::A.into()).is_down() {
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
		camera.set_orientation(self.rot);
		camera.set_position(self.pos + Vec3::from_y(1.4));
	}
}

