pub mod gl;
pub mod util;
pub mod input;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct RawStr (pub *const u8, pub usize);

impl<'a> Into<RawStr> for &'a str {
	fn into(self) -> RawStr {
		RawStr(self.as_ptr(), self.len())
	}
}


