
extern {
	pub fn console_log_raw(_: *const u8, _: usize);
	pub fn console_warn_raw(_: *const u8, _: usize);
	pub fn console_error_raw(_: *const u8, _: usize);

	pub fn canvas_width() -> i32;
	pub fn canvas_height() -> i32;

	pub fn math_random() -> f32;
}
