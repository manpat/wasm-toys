use std::fmt;

type OutputFunc = unsafe extern fn(*const u8, usize);

pub struct ConsoleIOBuffer (String, OutputFunc);

pub static mut CONSOLE_LOG_BUFFER: Option<ConsoleIOBuffer> = None;
pub static mut CONSOLE_WARN_BUFFER: Option<ConsoleIOBuffer> = None;
pub static mut CONSOLE_ERROR_BUFFER: Option<ConsoleIOBuffer> = None;

impl ConsoleIOBuffer {
	pub fn new(f: OutputFunc) -> Self {
		ConsoleIOBuffer (String::new(), f)
	}

	pub fn flush(&mut self) {
		let out = self.1;
		unsafe { out(self.0.as_ptr(), self.0.len()); }
		self.0.clear();
	}
}

impl fmt::Write for ConsoleIOBuffer {
	fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
		let mut it = s.split('\n');
		if let Some(s) = it.next() {
			self.0.push_str(s);
		}

		for s in it {
			self.flush();
			self.0.push_str(s);
		}

		Ok(())
	}
}

#[macro_export]
macro_rules! console_log {
	($fmt:expr $(,$arg:expr)*) => {{
		use std::fmt::Write;
		use $crate::io::{ConsoleIOBuffer, CONSOLE_LOG_BUFFER};
		use $crate::imports::util::console_log_raw;

		#[allow(unused_unsafe)]
		let buf = unsafe {
			CONSOLE_LOG_BUFFER.get_or_insert_with(|| ConsoleIOBuffer::new(console_log_raw))
		};

		write!(buf, $fmt $(,$arg)*).unwrap();
		buf.flush();
	}};
}

#[macro_export]
macro_rules! console_warn {
	($fmt:expr $(,$arg:expr)*) => {{
		use std::fmt::Write;
		use $crate::io::{ConsoleIOBuffer, CONSOLE_WARN_BUFFER};
		use $crate::imports::util::console_warn_raw;

		#[allow(unused_unsafe)]
		let buf = unsafe {
			CONSOLE_WARN_BUFFER.get_or_insert_with(|| ConsoleIOBuffer::new(console_warn_raw))
		};

		write!(buf, $fmt $(,$arg)*).unwrap();
		buf.flush();
	}};
}

#[macro_export]
macro_rules! console_error {
	($fmt:expr $(,$arg:expr)*) => {{
		use std::fmt::Write;
		use $crate::io::{ConsoleIOBuffer, CONSOLE_ERROR_BUFFER};
		use $crate::imports::util::console_error_raw;

		#[allow(unused_unsafe)]
		let buf = unsafe {
			CONSOLE_ERROR_BUFFER.get_or_insert_with(|| ConsoleIOBuffer::new(console_error_raw))
		};

		write!(buf, $fmt $(,$arg)*).unwrap();
		buf.flush();
	}};
}


