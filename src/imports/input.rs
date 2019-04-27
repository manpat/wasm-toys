

extern {
	pub fn init_input_listeners(passive: bool);

	pub fn request_pointer_lock();
	pub fn exit_pointer_lock();
}