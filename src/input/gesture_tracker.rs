use crate::prelude::*;
use crate::input::core::*;

#[derive(Debug)]
pub struct GestureTracker {
	client_size: Vec2,
	state: ButtonState,

	current_position: Vec2,
	initial_position: Vec2,
	delta: Vec2,
	distance_travelled: u32,

	hold_begin: Ticks,
	current_time: Ticks,

	drag_threshold: u32,
	hold_threshold: u32,
}


impl GestureTracker {
	pub fn new(drag_threshold: u32, hold_threshold: u32) -> Self {
		Self {
			client_size: Vec2::zero(),

			state: ButtonState::Up,
			current_position: Vec2::zero(),
			initial_position: Vec2::zero(),
			delta: Vec2::zero(),
			distance_travelled: 0,

			hold_begin: 0,
			current_time: 0,

			// TODO: deal with dpi?
			drag_threshold,
			hold_threshold,
		}
	}

	pub fn update(&mut self, input: &InputContext, viewport: Vec2i, time: Ticks) {
		self.client_size = viewport.to_vec2();
		self.current_time = time;

		let (new_state, position, delta) = get_primary_state(input);
		self.state = new_state;
		self.delta = delta.to_vec2();

		if self.state != ButtonState::Up {
			self.current_position = position.to_vec2();
		} else {
			self.distance_travelled = 0;
		}

		if self.state.is_pressed() {
			self.initial_position = position.to_vec2();
			self.distance_travelled = 0;
			self.hold_begin = time;

		} else if self.state.is_down() {
			self.distance_travelled = self.distance_travelled.saturating_add((delta.x.abs() + delta.y.abs()) as u32);
		}
	}

	pub fn press(&self) -> bool { self.state == ButtonState::DownRecent }
	pub fn release(&self) -> bool { self.state == ButtonState::UpRecent }
	pub fn down(&self) -> bool { self.state.is_down() }

	pub fn tap(&self) -> bool {
		self.state == ButtonState::UpRecent
			&& (self.current_time - self.hold_begin) < self.hold_threshold
			&& !self.dragging()
	}

	pub fn holding(&self) -> bool {
		self.state.is_down()
			&& (self.current_time - self.hold_begin) >= self.hold_threshold
			&& !self.dragging()
	}

	pub fn dragging(&self) -> bool {
		self.state != ButtonState::Up && self.distance_travelled >= self.drag_threshold
	}

	pub fn position(&self) -> Vec2 {
		(self.current_position / self.client_size * 2.0 - 1.0) * Vec2::new(1.0, -1.0)
	}

	pub fn initial_position(&self) -> Vec2 {
		(self.initial_position / self.client_size * 2.0 - 1.0) * Vec2::new(1.0, -1.0)
	}

	pub fn frame_delta(&self) -> Vec2 {
		self.delta / self.client_size * Vec2::new(1.0, -1.0)
	}

	pub fn drag_delta(&self) -> Vec2 {
		self.position() - self.initial_position()
	}
}


fn get_primary_state(ctx: &InputContext) -> (ButtonState, Vec2i, Vec2i) {
	let mut button_state = ButtonState::Up;
	let mut pos = Vec2i::zero();
	let mut delta = Vec2i::zero();

	if ctx.touch_mode {
		let primary_touch = ctx.touch_states.first()
			.filter(|s| s.index == 0);

		if let Some(state) = primary_touch {
			button_state = state.state;
			pos = state.pos;
			delta = state.frame_delta;
		}
	} else {
		let mouse_state = ctx.button_state(MouseButton::Left);
		if ctx.is_pointer_locked() || mouse_state != ButtonState::Up {
			button_state = mouse_state;
			pos = ctx.mouse_pos;
			delta += ctx.mouse_delta;
		}
	}

	(button_state, pos, delta)
}
