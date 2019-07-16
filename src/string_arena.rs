#![allow(unused_attributes)]

use std::{fmt, ops, str, slice};
use crate::*;

const INVALID_STRING_SIZE: usize = 0xFFFF_FFFF;


struct StringArena { buffer: Vec<u8>, used: usize }

impl StringArena {
	fn new() -> Self {
		StringArena { buffer: Vec::with_capacity(1024), used: 0 }
	}

	fn allocate(&mut self, size: usize) -> *mut u8 {
		let start = self.buffer.len();
		self.buffer.resize(start + size + 4, 0);
		self.used += size + 4;

		unsafe {
			let size_start = self.buffer.as_mut_ptr().add(start);
			let data_start = size_start.add(4);

			// console_log!("allocated {}B from {:?}", size + 4, size_start);
			
			(size_start as *mut u32).write(size as u32);
			data_start
		}
	}

	fn free(&mut self, ptr: *const u8) {
		let buf = self.buffer.as_mut_ptr();
		unsafe {
			let pos = ptr.offset_from(buf);
			assert!(pos >= 0, "tried to free non-owned temporary string at {:?}!", ptr);

			let size_ptr = ptr.sub(4) as *mut u32;
			let size = size_ptr.read() as usize + 4;

			if pos >= self.buffer.len() as isize && size != 4 {
				console_error!("tried to free non-owned temporary string at {:?}!", ptr);
				return;
			}
			size_ptr.write(INVALID_STRING_SIZE as u32);

			// console_log!("freed {}B from {:?}", size, size_ptr);

			assert!(size != INVALID_STRING_SIZE, "temporary string double free");
			assert!(self.used >= size, "temporary string arena has been corrupted!");

			self.used -= size;

			if self.used == 0 {
				// console_log!("temp string arena empty");
				self.buffer.clear();
			}
		}
	}
}


#[repr(C)]
#[derive(Debug)]
pub struct JSString (*const u8);

impl JSString {
	pub fn as_str(&self) -> &str {
		unsafe {
			assert!(!self.0.is_null(), "Attempting to get JSString as &str");

			let size = self.len();
			let s = slice::from_raw_parts(self.0, size);
			str::from_utf8(s).unwrap()
		}
	}

	pub fn len(&self) -> usize {
		unsafe {
			assert!(!self.0.is_null(), "Attempting to get length of null JSString");

			let size = (self.0.sub(4) as *const u32).read() as usize;
			assert!(size != INVALID_STRING_SIZE, "Attempting to read deallocated temporary string");
			size
		}
	}
}

impl ops::Deref for JSString {
	type Target = str;
	fn deref(&self) -> &str { self.as_str() }
}

impl ops::Drop for JSString {
	fn drop(&mut self) {
		free_str_space(self.0);
	}
}

impl fmt::Display for JSString {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		self.as_str().fmt(f)
	}
}


static mut TEMPORARY_STRING_ARENA: Option<StringArena> = None;

fn get_temp_string_arena() -> &'static mut StringArena {
	unsafe { TEMPORARY_STRING_ARENA.get_or_insert_with(|| StringArena::new()) }
}

// exports

#[no_mangle]
pub fn allocate_str_space(size: usize) -> *mut u8 {
	let arena = get_temp_string_arena();
	arena.allocate(size)
}

#[no_mangle]
pub fn free_str_space(ptr: *const u8) {
	let arena = get_temp_string_arena();
	arena.free(ptr);
}
