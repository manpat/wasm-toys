use common::math::*;
use crate::input::InputContext;
use crate::imports::gl;

pub type Ticks = u32;
pub type EngineResult<T> = Result<T, failure::Error>;

pub struct Engine {
	pub client: Box<dyn EngineClient>,

	pub input: InputContext,

	pub viewport: Vec2i,
	pub time_ticks: Ticks,
}

impl Engine {
	pub fn new<C: EngineClient + 'static>(client: C) -> Self {
		unsafe {
			gl::enable_attribute(0);
			gl::enable_attribute(1);

			gl::enable(gl::Capability::DepthTest);
			gl::enable(gl::Capability::Blend);
			gl::blend_func(gl::BlendFactor::One, gl::BlendFactor::OneMinusSrcAlpha);

			let input = InputContext::new(client.uses_passive_input());

			Engine {
				client: box client,
				input,

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

		let upd_ctx = UpdateContext {
			ticks: self.time_ticks,
			viewport: self.viewport,
			input: &self.input,
		};

		self.client.update(upd_ctx);

		self.input.clear_frame_state();
		self.time_ticks = self.time_ticks.wrapping_add(1);
	}
}

pub struct UpdateContext<'eng> {
	pub ticks: Ticks,
	pub viewport: Vec2i,
	pub input: &'eng InputContext,
}

pub trait EngineClient {
	fn uses_passive_input(&self) -> bool { true }

	fn init(&mut self) {}
	fn update(&mut self, _: UpdateContext) {}
}

