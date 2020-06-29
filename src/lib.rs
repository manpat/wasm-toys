#![feature(panic_info_message, box_syntax, ptr_offset_from)]
#![feature(clamp)]
#![feature(nll)]
#![deny(rust_2018_idioms, future_incompatible)]

pub mod prelude;

pub mod imports;

#[macro_use]
pub mod io;
pub mod string_arena;

pub mod exports;

pub mod engine;
pub mod input;
pub mod graphics;

pub use engine::{EngineClient, UpdateContext, Ticks, EngineResult};
pub const DT: f32 = 1.0/60.0;

use std::cell::{RefCell, Ref, RefMut};

// RefCell is an attempt at guarding against mutable ref aliasing across the js/wasm boundary
static mut ENGINE: Option<RefCell<engine::Engine>> = None;

pub fn init_engine<F: FnOnce() -> C, C: EngineClient + 'static>(client: F) {
	std::panic::set_hook(box |panic_info| {
		if let Some(loc) = panic_info.location() {
			console_error!("panic at {}:{}!", loc.file(), loc.line());
		} 

		if let Some(msg) = panic_info.message() {
			console_error!("{}", msg);
		}

		if let Some(msg) = panic_info.payload().downcast_ref::<&str>() {
			console_error!("{}", msg);
		}
	});

	exports::force_linkage();

	unsafe {
		ENGINE = Some(RefCell::new(engine::Engine::new(client())));
	}

	get_engine_mut().client.init();
}

pub fn get_engine() -> Ref<'static, engine::Engine> {
	unsafe {
		ENGINE.as_ref().unwrap().borrow()
	}
}

pub fn get_engine_mut() -> RefMut<'static, engine::Engine> {
	unsafe {
		ENGINE.as_ref().unwrap().borrow_mut()
	}
}
