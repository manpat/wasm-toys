use common::math::*;
use crate::input::{InputContext, GestureTracker};
use crate::imports::gl;

pub type Ticks = u32;
pub type EngineResult<T> = Result<T, failure::Error>;

pub struct Engine {
	pub client: Box<dyn EngineClient>,

	pub input_context: InputContext,
	pub gesture_tracker: GestureTracker,

	pub viewport: Vec2i,
	pub time_ticks: Ticks,
}

impl Engine {
	pub fn new<C: EngineClient + 'static>(client: C) -> Self {
		unsafe {
			gl::enable_attribute(0);

			gl::enable(gl::Capability::DepthTest);
			gl::enable(gl::Capability::Blend);
			gl::blend_func(gl::BlendFactor::One, gl::BlendFactor::OneMinusSrcAlpha);

			let mut input_context = InputContext::new(client.uses_passive_input());
			input_context.enable_pointer_lock(client.captures_input());

			let drag_threshold = client.drag_threshold().unwrap_or(0);
			let hold_threshold = client.hold_threshold().unwrap_or(std::u32::MAX);

			Engine {
				client: box client,
				input_context,
				gesture_tracker: GestureTracker::new(drag_threshold, hold_threshold),

				viewport: Vec2i::new(0, 0),
				time_ticks: 0,
			}
		}
	}

	pub fn update(&mut self, _time: f64) {
		unsafe {
			let Vec2i{x, y} = self.viewport;
			gl::viewport(0, 0, x, y);
		}

		self.gesture_tracker.update(&self.input_context, self.viewport, self.time_ticks);

		let upd_ctx = UpdateContext {
			ticks: self.time_ticks,
			viewport: self.viewport,
			input: &self.gesture_tracker,
			input_raw: &self.input_context,
		};

		self.client.update(upd_ctx);

		self.input_context.clear_frame_state();
		self.time_ticks = self.time_ticks.wrapping_add(1);
	}
}

pub struct UpdateContext<'eng> {
	pub ticks: Ticks,
	pub viewport: Vec2i,
	pub input: &'eng GestureTracker,
	pub input_raw: &'eng InputContext,
}

pub trait EngineClient {
	fn uses_passive_input(&self) -> bool { true }
	fn captures_input(&self) -> bool { false }
	fn drag_threshold(&self) -> Option<u32> { Some(5) }
	fn hold_threshold(&self) -> Option<Ticks> { None } // Holding disabled by default

	fn init(&mut self) {}
	fn update(&mut self, _: UpdateContext<'_>) {}
}

