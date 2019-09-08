#![allow(unused_attributes)]

use common::*;

use crate::get_engine_mut;
use crate::string_arena::JSString;
use crate::imports::user;
use crate::input::*;

// HACK - without this, this whole module will get dropped
pub fn force_linkage() {}

// core

#[no_mangle]
pub fn internal_update(t: f64) {
	// This is here and not in Engine::update to avoid potential reentrancy
	unsafe { user::user_update(); }
	get_engine_mut().update(t);
}

#[no_mangle]
pub fn internal_update_viewport(w: i32, h: i32) {
	get_engine_mut().viewport = Vec2i::new(w,h);
}



// allocation

#[no_mangle]
pub fn internal_allocate_i32_vec(n: usize) -> *mut i32 {
	let mut s = vec![0; n].into_boxed_slice();
	let ptr = s.as_mut_ptr();
	std::mem::forget(s);
	ptr
}

// input

#[no_mangle]
pub fn internal_handle_key_down(key_code: JSString) -> bool {
	let code = KeyCode::from_js_code(key_code);
	if code.is_none() { return false }

	let code = code.unwrap();
	get_engine_mut().input_context.register_keydown(code);
	code.always_consume() // TODO: only if view has focus
}

#[no_mangle]
pub fn internal_handle_key_up(key_code: JSString) -> bool {
	let code = KeyCode::from_js_code(key_code);
	if code.is_none() { return false }

	let code = code.unwrap();
	get_engine_mut().input_context.register_keyup(code);
	code.always_consume() // TODO: only if view has focus
}

#[no_mangle]
pub fn internal_handle_mouse_down(mb: MouseButton, x: i32, y: i32) -> bool {
	// TODO: this needs to not register clicks outside of views when not pointer locked
	get_engine_mut().input_context.register_mousedown(mb, x, y);
	true
}

#[no_mangle]
pub fn internal_handle_mouse_up(mb: MouseButton, x: i32, y: i32) -> bool {
	// TODO: this needs to not register clicks outside of views when not pointer locked
	get_engine_mut().input_context.register_mouseup(mb, x, y);
	true
}

#[no_mangle]
pub fn internal_handle_mouse_move(x: i32, y: i32, dx: i32, dy: i32) -> bool {
	get_engine_mut().input_context.mouse_pos = Vec2i::new(x, y);
	get_engine_mut().input_context.mouse_delta = Vec2i::new(dx, dy);
	false
}

#[no_mangle]
pub fn internal_handle_touch_down(id: i32, x: i32, y: i32) -> bool {
	get_engine_mut().input_context.register_touchdown(id, x, y);
	true
}

#[no_mangle]
pub fn internal_handle_touch_up(id: i32, x: i32, y: i32) -> bool {
	get_engine_mut().input_context.register_touchup(id, x, y);
	true
}

#[no_mangle]
pub fn internal_handle_touch_move(id: i32, x: i32, y: i32) -> bool {
	get_engine_mut().input_context.register_touchmove(id, x, y);
	false
}

#[no_mangle]
pub fn internal_handle_focus_gain() {
	get_engine_mut().input_context.reset_inputs();
}

#[no_mangle]
pub fn internal_handle_focus_loss() {
	get_engine_mut().input_context.reset_inputs();
}

#[no_mangle]
pub fn internal_notify_pointer_lock_change(enabled: bool) {
	get_engine_mut().input_context.register_pointer_lock_change(enabled);
}


#[no_mangle]
pub fn engine_enable_pointer_lock(e: bool) {
	get_engine_mut().input_context.enable_pointer_lock(e);
}
