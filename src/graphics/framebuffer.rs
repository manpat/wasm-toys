use crate::prelude::*;
use crate::imports::gl;
use std::marker::PhantomData;

pub struct Framebuffer {
	gl_handle: gl::FramebufferID,
	renderbuffer_handle: gl::RenderbufferID,
	color_tex: Texture,
}

impl Framebuffer {
	pub fn new() -> Self {
		Framebuffer {
			gl_handle: unsafe{ gl::create_framebuffer() },
			renderbuffer_handle: unsafe{ gl::create_renderbuffer() },
			color_tex: TextureBuilder::new().rgba8().build(),
		}
	}

	#[must_use]
	pub fn bind(&self) -> FramebufferBindGuard<'_> {
		FramebufferBindGuard::new(self.gl_handle, self.size())
	}

	pub fn size(&self) -> Vec2i {
		self.color_tex.size()
	}

	pub fn color_texture(&self) -> &Texture {
		&self.color_tex
	}

	pub fn resize(&mut self, s: Vec2i) {
		if s == self.size() { return }

		unsafe {
			gl::delete_framebuffer(self.gl_handle);
			self.gl_handle = gl::create_framebuffer();
			gl::bind_framebuffer(self.gl_handle);

			self.color_tex.reserve(s);
			gl::framebuffer_texture_2d(self.color_tex.gl_handle);

			gl::bind_renderbuffer(self.renderbuffer_handle);
			gl::renderbuffer_depth_storage(s.x, s.y);
			gl::framebuffer_renderbuffer(self.renderbuffer_handle);
			gl::bind_renderbuffer(gl::RenderbufferID(0));

			gl::bind_framebuffer(gl::FramebufferID(0));
		}
	}
}



pub struct FramebufferBindGuard<'fb> {
	prev_binding: Option<gl::FramebufferID>,
	prev_viewport: [i32; 4],

	phantom: PhantomData<&'fb Framebuffer>,
}

impl<'fb> FramebufferBindGuard<'fb> {
	fn new(new_binding: gl::FramebufferID, viewport: Vec2i) -> Self {
		unsafe {
			let prev_binding = gl::get_bound_framebuffer();

			let mut prev_viewport = [0; 4];
			gl::get_viewport(prev_viewport.as_mut_ptr(), prev_viewport.len());

			if prev_binding != new_binding {
				gl::viewport(0, 0, viewport.x, viewport.y);
				gl::bind_framebuffer(new_binding);

				FramebufferBindGuard {
					prev_binding: Some(prev_binding),
					prev_viewport,
					phantom: PhantomData
				}

			} else {
				FramebufferBindGuard { prev_binding: None, prev_viewport, phantom: PhantomData }
			}
		}
	}
}

impl<'fb> Drop for FramebufferBindGuard<'fb> {
	fn drop(&mut self) {
		if let Some(prev_binding) = self.prev_binding {
			unsafe{
				gl::bind_framebuffer(prev_binding);

				gl::viewport(
					self.prev_viewport[0], self.prev_viewport[1],
					self.prev_viewport[2], self.prev_viewport[3]
				);
			}
		}
	}
}