use common::math::Vec2i;
use crate::string_arena::JSString;
use crate::imports::input;

use std::convert::Into;


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


#[repr(usize)]
#[derive(Copy, Clone, Debug)]
pub enum Intent {
	Up, Down, Left, Right,
	Primary, Secondary,

	Count,
}


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


pub struct InputContext {
	key_states: [ButtonState; KeyCode::Count as usize],
	mb_states: [ButtonState; MouseButton::Count as usize],
	pub mouse_pos: Vec2i,
	pub mouse_delta: Vec2i,

	is_pointer_locked: bool,
	should_pointer_lock: bool,
	pub pointer_lock_allowed: bool,

	intent_map: [Vec<Button>; Intent::Count as usize],
}


impl InputContext {
	pub fn new() -> InputContext {
		InputContext {
			key_states: [ButtonState::Up; KeyCode::Count as usize],
			mb_states: [ButtonState::Up; MouseButton::Count as usize],
			mouse_pos: Vec2i::zero(),
			mouse_delta: Vec2i::zero(),

			is_pointer_locked: false,
			should_pointer_lock: false,
			pointer_lock_allowed: false,
			
			intent_map: [
				// Up
				vec![
					KeyCode::W.into(),
					KeyCode::Up.into(),
				],
				// Down
				vec![
					KeyCode::S.into(),
					KeyCode::Down.into(),
				],
				// Left
				vec![
					KeyCode::A.into(),
					KeyCode::Left.into(),
				],
				// Right
				vec![
					KeyCode::D.into(),
					KeyCode::Right.into(),
				],

				// Primary
				vec![ KeyCode::F.into(), MouseButton::Left.into() ],

				// Secondary
				vec![ MouseButton::Right.into() ],
			] 
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
	}

	pub fn reset_inputs(&mut self) {
		for state in self.key_states.iter_mut() {
			*state = ButtonState::Up;
		}

		for state in self.mb_states.iter_mut() {
			*state = ButtonState::Up;
		}
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

	pub fn intent_state(&self, intent: Intent) -> ButtonState {
		if intent as usize >= Intent::Count as usize {
			console_error!("Tried to get state of invalid Intent '{}'", intent as usize);
			return ButtonState::Up;
		}

		let mut acc_state = ButtonState::Up;

		for b in self.intent_map[intent as usize].iter() {
			match self.button_state(*b) {
				ButtonState::Down => { acc_state = ButtonState::Down }

				ButtonState::DownRecent => if acc_state.is_up() {
					acc_state = ButtonState::DownRecent
				}
				ButtonState::UpRecent => if acc_state.is_up() {
					acc_state = ButtonState::UpRecent
				}
				_ => {}
			}
		}

		acc_state
	}

	pub fn enable_pointer_lock(&mut self, e: bool) {
		self.should_pointer_lock = e;

		// Pointer lock doesn't require a gesture to release
		if self.is_pointer_locked && !self.should_pointer_lock {
			unsafe { input::exit_pointer_lock(); }
		}
	}

	pub fn is_pointer_locked(&self) -> bool { self.is_pointer_locked }

	pub fn register_keydown(&mut self, code: KeyCode) {
		let s = &mut self.key_states[code as usize];
		if s.is_up() { *s = ButtonState::DownRecent }

		if !code.is_modifier() {
			self.try_set_pointer_lock();
		}
	}

	pub fn register_keyup(&mut self, code: KeyCode) {
		let s = &mut self.key_states[code as usize];
		if s.is_down() { *s = ButtonState::UpRecent }
		
		if !code.is_modifier() {
			self.try_set_pointer_lock();
		}
	}

	pub fn register_mousedown(&mut self, mb: MouseButton, x: i32, y: i32) {
		let s = &mut self.mb_states[mb as usize];
		if s.is_up() { *s = ButtonState::DownRecent }
		self.mouse_pos = Vec2i::new(x, y);
		self.try_set_pointer_lock();
	}

	pub fn register_mouseup(&mut self, mb: MouseButton, x: i32, y: i32) {
		let s = &mut self.mb_states[mb as usize];
		if s.is_down() { *s = ButtonState::UpRecent }
		self.mouse_pos = Vec2i::new(x, y);
		self.try_set_pointer_lock();
	}

	pub fn register_pointer_lock_change(&mut self, enabled: bool) {
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
