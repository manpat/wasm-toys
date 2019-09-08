use common::math::Vec2i;
use crate::string_arena::JSString;
use crate::imports::input;


#[repr(usize)]
#[derive(Copy, Clone, Debug)]
pub enum KeyCode {
	Left, Right, Up, Down,
	W, A, S, D,
	Q, E, F,

	Space, Enter, Escape,
	Shift, Ctrl, Alt,

	F1,

	Count
}

#[repr(usize)]
#[derive(Copy, Clone, Debug)]
pub enum MouseButton { Left, Middle, Right,  Count }


#[derive(Copy, Clone, Debug)]
pub enum Button {
	Key(KeyCode),
	Mouse(MouseButton),
}

impl Into<Button> for KeyCode {
	fn into(self) -> Button { Button::Key(self) }
}
impl Into<Button> for MouseButton {
	fn into(self) -> Button { Button::Mouse(self) }
}


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ButtonState {
	Up, Down,
	UpRecent, DownRecent
}


#[derive(Copy, Clone)]
pub struct TouchInstance {
	pub(crate) id: i32,
	pub(crate) index: u32,
	pub(crate) pos: Vec2i,
	pub(crate) frame_delta: Vec2i,
	pub(crate) state: ButtonState,
}


pub struct InputContext {
	key_states: [ButtonState; KeyCode::Count as usize],
	mb_states: [ButtonState; MouseButton::Count as usize],
	pub(crate) touch_states: Vec<TouchInstance>,

	// TODO: probably improve this
	pub(crate) mouse_pos: Vec2i,
	pub(crate) mouse_delta: Vec2i,

	is_pointer_locked: bool,
	should_pointer_lock: bool,
	pub(crate) pointer_lock_allowed: bool,

	pub(crate) touch_mode: bool,
}


impl InputContext {
	pub fn new(uses_passive_input: bool) -> InputContext {
		unsafe {
			input::init_input_listeners(uses_passive_input);
		}

		InputContext {
			key_states: [ButtonState::Up; KeyCode::Count as usize],
			mb_states: [ButtonState::Up; MouseButton::Count as usize],
			touch_states: Vec::new(),

			mouse_pos: Vec2i::zero(),
			mouse_delta: Vec2i::zero(),

			is_pointer_locked: false,
			should_pointer_lock: false,
			pointer_lock_allowed: false,

			touch_mode: false,
		}
	}

	pub fn clear_frame_state(&mut self) {
		self.mouse_delta = Vec2i::zero();

		for state in self.key_states.iter_mut() {
			*state = state.recent_flag_cleared();
		}

		for state in self.mb_states.iter_mut() {
			*state = state.recent_flag_cleared();
		}

		for state in self.touch_states.iter_mut() {
			state.frame_delta = Vec2i::zero();
			state.state = state.state.recent_flag_cleared();
		}

		self.touch_states.retain(|s| s.state != ButtonState::Up);
	}

	pub fn reset_inputs(&mut self) {
		for state in self.key_states.iter_mut() {
			*state = ButtonState::Up;
		}

		for state in self.mb_states.iter_mut() {
			*state = ButtonState::Up;
		}

		self.touch_states.clear();
	}

	pub fn button_state(&self, b: Button) -> ButtonState {
		match b {
			Button::Key(k) => {
				if k as usize >= KeyCode::Count as usize {
					console_error!("Tried to get state of invalid KeyCode '{}'", k as usize);
					return ButtonState::Up;
				}
				
				self.key_states[k as usize]
			}

			Button::Mouse(m) => {
				if m as usize >= MouseButton::Count as usize {
					console_error!("Tried to get state of invalid MouseButton '{}'", m as usize);
					return ButtonState::Up;
				}

				self.mb_states[m as usize]
			}
		}
	}

	// TODO: Use ButtonState here
	pub fn primary_down(&self) -> bool {
		if self.touch_mode {
			self.touch_states.first()
				.map(|s| s.index == 0)
				.unwrap_or(false)
		} else {
			self.button_state(MouseButton::Left.into())
				.is_down()
		}
	}

	pub fn primary_delta(&self) -> Vec2i {
		if self.touch_mode {
			self.touch_states.first()
				.filter(|s| s.index == 0)
				.map(|s| s.frame_delta)
				.unwrap_or(Vec2i::zero())
		} else {
			self.mouse_delta
		}
	}


	pub fn enable_pointer_lock(&mut self, e: bool) {
		self.should_pointer_lock = e;

		// Pointer lock doesn't require a gesture to release
		if self.is_pointer_locked && !self.should_pointer_lock {
			unsafe { input::exit_pointer_lock(); }
		}
	}

	pub fn is_pointer_locked(&self) -> bool { self.is_pointer_locked }

	pub(crate) fn register_keydown(&mut self, code: KeyCode) {
		let s = &mut self.key_states[code as usize];
		if s.is_up() { *s = ButtonState::DownRecent }

		if !code.is_modifier() {
			self.try_set_pointer_lock();
		}
	}

	pub(crate) fn register_keyup(&mut self, code: KeyCode) {
		let s = &mut self.key_states[code as usize];
		if s.is_down() { *s = ButtonState::UpRecent }
		
		if !code.is_modifier() {
			self.try_set_pointer_lock();
		}
	}

	pub(crate) fn register_mousedown(&mut self, mb: MouseButton, x: i32, y: i32) {
		self.touch_mode = false;

		let s = &mut self.mb_states[mb as usize];
		if s.is_up() { *s = ButtonState::DownRecent }
		self.mouse_pos = Vec2i::new(x, y);
		self.try_set_pointer_lock();
	}

	pub(crate) fn register_mouseup(&mut self, mb: MouseButton, x: i32, y: i32) {
		let s = &mut self.mb_states[mb as usize];
		if s.is_down() { *s = ButtonState::UpRecent }
		self.mouse_pos = Vec2i::new(x, y);
		self.try_set_pointer_lock();
	}

	pub(crate) fn register_touchdown(&mut self, id: i32, x: i32, y: i32) {
		self.touch_mode = true;

		// TODO: check if id already exists and move to end

		let index = self.touch_states.len() as u32;
		self.touch_states.push(TouchInstance {
			id,
			pos: Vec2i::new(x, y),
			frame_delta: Vec2i::zero(),
			index,
			state: ButtonState::DownRecent,
		});
	}

	pub(crate) fn register_touchup(&mut self, id: i32, x: i32, y: i32) {
		if let Some(state) = self.touch_states.iter_mut().find(|s| s.id == id) {
			let new_pos = Vec2i::new(x, y);
			let diff = new_pos - state.pos;

			state.pos = new_pos;
			state.frame_delta += diff;
			state.state = ButtonState::UpRecent;
		}
	}

	pub(crate) fn register_touchmove(&mut self, id: i32, x: i32, y: i32) {
		if let Some(state) = self.touch_states.iter_mut().find(|s| s.id == id) {
			let new_pos = Vec2i::new(x, y);
			let diff = new_pos - state.pos;

			state.pos = new_pos;
			state.frame_delta += diff;
		}
	}

	pub(crate) fn register_pointer_lock_change(&mut self, enabled: bool) {
		self.is_pointer_locked = enabled;
	}

	fn try_set_pointer_lock(&self) {
		// Pointer lock has to be requested during user input
		if self.should_pointer_lock && self.pointer_lock_allowed && !self.is_pointer_locked {
			unsafe { input::request_pointer_lock(); }
		}
	}
}


impl ButtonState {
	pub fn is_down(self) -> bool {
		self == ButtonState::Down || self == ButtonState::DownRecent
	}

	pub fn is_up(self) -> bool {
		self == ButtonState::Up || self == ButtonState::UpRecent
	}

	pub fn is_pressed(self) -> bool {
		self == ButtonState::DownRecent
	}

	pub fn is_released(self) -> bool {
		self == ButtonState::UpRecent
	}

	pub fn recent_flag_cleared(self) -> ButtonState {
		match self {
			ButtonState::UpRecent => { ButtonState::Up }
			ButtonState::DownRecent => { ButtonState::Down }
			_ => self
		}
	}
}


impl KeyCode {
	pub fn from_js_code(s: JSString) -> Option<KeyCode> {
		match s.as_str() {
			"ArrowLeft" => Some(KeyCode::Left),
			"ArrowRight" => Some(KeyCode::Right),
			"ArrowUp" => Some(KeyCode::Up),
			"ArrowDown" => Some(KeyCode::Down),

			"KeyW" => Some(KeyCode::W),
			"KeyA" => Some(KeyCode::A),
			"KeyS" => Some(KeyCode::S),
			"KeyD" => Some(KeyCode::D),

			"KeyQ" => Some(KeyCode::Q),
			"KeyE" => Some(KeyCode::E),
			"KeyF" => Some(KeyCode::F),

			"Enter" => Some(KeyCode::Enter),
			"Space" => Some(KeyCode::Space),
			"Escape" => Some(KeyCode::Escape),

			"ControlLeft" | "ControlRight" => Some(KeyCode::Ctrl),
			"ShiftLeft" | "ShiftRight" => Some(KeyCode::Shift),
			"AltLeft" | "AltRight" => Some(KeyCode::Alt),

			"F1" => Some(KeyCode::F1),

			_ => None
		}
	}

	pub fn is_modifier(&self) -> bool {
		match *self {
			KeyCode::Ctrl
			| KeyCode::Shift
			| KeyCode::Alt => true,

			_ => false
		}
	}

	pub fn always_consume(&self) -> bool {
		match *self {
			// We don't want to scroll
			KeyCode::Left
			| KeyCode::Right
			| KeyCode::Up
			| KeyCode::Down
			| KeyCode::Space => true,

			// No help for you
			KeyCode::F1 => true,

			_ => false
		}
	}
}
