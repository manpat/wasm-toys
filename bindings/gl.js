"use strict";

export let gl_module = {
	programs: [null],
	shaders: [null],
	buffers: [null],

	framebuffers: [null],
	renderbuffers: [null],
	textures: [null],
	named_textures: {},


	init: function(engine) {
		this.engine = engine;
		this.context = this.create_context(engine.canvas, { stencil: true });
	},


	create_context: function(canvas, params) {
		try {
			let gl = canvas.getContext("experimental-webgl", params)
				|| canvas.getContext("webgl", params);

			if (typeof gl === "object")
				return gl;

		} catch(error) {
			console.error(error);
		}

		throw "WebGL not supported";
	},


	get_named_texture: function(id) {
		return this.named_textures[id] || 0;
	},


	load_texture: function(el) {
		let gl = this.context;

		if (typeof el !== "object") {
			throw `Trying to load texture with invalid object`;
		}

		let tex = gl.createTexture();
		gl.bindTexture(gl.TEXTURE_2D, tex);
		gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, el);

		gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
		gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);

		gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
		gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

		this.textures.push(tex);
		return this.textures.length;
	},

	load_named_texture: function(id, el) {
		let tex_id = this.named_textures[id];

		if (typeof tex_id !== 'undefined') {
			return tex_id;
		}

		if (typeof el !== 'object') {
			throw `Trying to load named texture ${id} from invalid object`;
		}

		tex_id = this.load_texture(el);
		this.named_textures[id] = tex_id;

		return tex_id
	},


	imports: function(engine) {
		return {
			// General state stuff
			viewport: (x,y,w,h) => this.context.viewport(x, y, w, h),
			scissor: (x,y,w,h) => this.context.scissor(x, y, w, h),

			get_viewport: (ptr, len) => {
				let buf = this.engine.wasm.heap_memory_view_u32(ptr, len);
				let arr = this.context.getParameter(this.context.VIEWPORT);
				buf.set(arr);
			},

			clear_color: (r,g,b,a) => this.context.clearColor(r,g,b,a),
			clear: (x) => this.context.clear(x),

			enable: (e) => this.context.enable(e),
			disable: (e) => this.context.disable(e),

			blend_func: (s, d) => this.context.blendFunc(s, d),
			
			// Draw stuff
			draw_arrays: (draw_mode, start, count) => this.context.drawArrays(draw_mode, start, count),
			draw_elements: (draw_mode, count, type, offset) => this.context.drawElements(draw_mode, count, type, offset),


			// Buffer stuff
			create_buffer: () => {
				this.buffers.push(this.context.createBuffer());
				return this.buffers.length;
			},

			bind_buffer: (target, id) => {
				let buf = this.buffers[id-1] || null;
				this.context.bindBuffer(target, buf);
			},

			upload_buffer_data: (target, ptr, len) => {
				let buf = this.engine.wasm.heap_memory_view(ptr, len);
				this.context.bufferData(target, buf, this.context.STATIC_DRAW);
			},

			vertex_attrib_pointer: (attrib, components, component_type, normalize, stride, offset) => {
				this.context.vertexAttribPointer(attrib, components, component_type, normalize, stride, offset);
			},

			enable_attribute: (attrib) => this.context.enableVertexAttribArray(attrib),
			disable_attribute: (attrib) => this.context.disableVertexAttribArray(attrib),


			// Texture stuff
			create_texture: () => {
				this.textures.push(this.context.createTexture());
				return this.textures.length;
			},

			bind_texture: (id) => {
				let texture = this.textures[id-1] || null;
				this.context.bindTexture(this.context.TEXTURE_2D, texture);
			},

			active_texture: (id) => this.context.activeTexture(this.context.TEXTURE0 + id),

			upload_image_data: (w, h, format, type, ptr, len) => {
				let buf = this.engine.wasm.heap_memory_view(ptr, len);
				this.context.texImage2D(this.context.TEXTURE_2D, 0, format, w, h, 0,
					format, type, buf);
			},

			tex_parameter: (param, value) => {
				this.context.texParameteri(this.context.TEXTURE_2D, param, value);
			},


			// Framebuffer stuff
			create_framebuffer: () => {
				this.framebuffers.push(this.context.createFramebuffer());
				return this.framebuffers.length;
			},

			delete_framebuffer: (fb_id) => {
				let fb = this.framebuffers[fb_id-1] || null;
				if (fb) {
					this.context.deleteFramebuffer(fb);
					this.framebuffers[fb_id-1] = null;
				}
			},

			bind_framebuffer: (fb_id) => {
				let fb = this.framebuffers[fb_id-1] || null;
				this.context.bindFramebuffer(this.context.FRAMEBUFFER, fb);
			},

			get_bound_framebuffer: () => {
				let binding = this.context.getParameter(this.context.FRAMEBUFFER_BINDING);
				if (!binding) {
					return 0;
				}

				let pos = this.framebuffers.indexOf(binding);
				return pos+1;
			},

			framebuffer_texture_2d: (tex_id) => {
				let texture = this.textures[tex_id-1] || null;

				this.context.framebufferTexture2D(
					this.context.FRAMEBUFFER,
					this.context.COLOR_ATTACHMENT0, 
					this.context.TEXTURE_2D, texture, 0
				);
			},

			framebuffer_renderbuffer: (rb_id) => {
				let renderbuffer = this.renderbuffers[rb_id-1] || null;

				this.context.framebufferRenderbuffer(
					this.context.FRAMEBUFFER,
					this.context.DEPTH_ATTACHMENT, 
					this.context.RENDERBUFFER, renderbuffer
				);
			},


			// Renderbuffer stuff
			create_renderbuffer: () => {
				this.renderbuffers.push(this.context.createRenderbuffer());
				return this.renderbuffers.length;
			},

			delete_renderbuffer: (rb_id) => {
				let fb = this.renderbuffers[rb_id-1] || null;
				if (fb) {
					this.context.deleteRenderbuffer(fb);
					this.renderbuffers[rb_id-1] = null;
				}
			},

			bind_renderbuffer: (rb_id) => {
				let fb = this.renderbuffers[rb_id-1] || null;
				this.context.bindRenderbuffer(this.context.RENDERBUFFER, fb);
			},

			renderbuffer_depth_storage: (w, h) => {
				this.context.renderbufferStorage(
					this.context.RENDERBUFFER, this.context.DEPTH_COMPONENT16,
					w, h
				);
			},


			// Shader stuff
			create_shader_program: () => {
				this.programs.push(this.context.createProgram());
				return this.programs.length;
			},

			create_shader: (type, src_ptr, src_len) => {
				let sh;
				switch (type) {
					case 0: sh = this.context.createShader(this.context.VERTEX_SHADER); break; 
					case 1: sh = this.context.createShader(this.context.FRAGMENT_SHADER); break; 
				}

				this.context.shaderSource(sh, this.engine.wasm.rust_str_to_js(src_ptr, src_len));
				this.context.compileShader(sh);

				let info = this.context.getShaderInfoLog(sh);
				if (info.length > 0) {
					console.error(info.slice(0, -1));
				}

				this.shaders.push(sh);
				return this.shaders.length;
			},

			bind_attrib_location: (program_id, name_ptr, name_len, idx) => {
				let program = this.programs[program_id-1] || null;
				let name = this.engine.wasm.rust_str_to_js(name_ptr, name_len);
				this.context.bindAttribLocation(program, idx, name);
			},

			link_program: (program_id, vert_id, frag_id) => {
				let program = this.programs[program_id-1] || null;
				let vert = this.shaders[vert_id-1] || null;
				let frag = this.shaders[frag_id-1] || null;

				this.context.attachShader(program, vert);
				this.context.attachShader(program, frag);
				this.context.linkProgram(program);

				let info = this.context.getProgramInfoLog(program);
				if (info.length > 0) {
					console.error(info.slice(0, -1));
				}
			},

			use_program: (program_id) => {
				let program = this.programs[program_id-1] || null;
				this.context.useProgram(program);
			},

			stencil_func: (condition, reference, mask) => this.context.stencilFunc(condition, reference, mask),
			stencil_op: (stencil_fail, depth_fail, pass) => this.context.stencilOp(stencil_fail, depth_fail, pass),
			color_mask: (r,g,b,a) => this.context.colorMask(r,g,b,a),
			depth_mask: (enabled) => this.context.depthMask(enabled),
			stencil_mask: (bits) => this.context.stencilMask(bits),

			set_uniform_int_raw: (program_id, name_ptr, name_len, i) => {
				let program = this.programs[program_id-1] || null;
				let u_name = this.engine.wasm.rust_str_to_js(name_ptr, name_len);

				let loc = this.context.getUniformLocation(program, u_name);
				this.context.uniform1i(loc, i);
			},

			set_uniform_f32_raw: (program_id, name_ptr, name_len, f) => {
				let program = this.programs[program_id-1] || null;
				let u_name = this.engine.wasm.rust_str_to_js(name_ptr, name_len);

				let loc = this.context.getUniformLocation(program, u_name);
				this.context.uniform1f(loc, f);
			},

			set_uniform_vec4_raw: (program_id, name_ptr, name_len, x,y,z,w) => {
				let program = this.programs[program_id-1] || null;
				let u_name = this.engine.wasm.rust_str_to_js(name_ptr, name_len);

				let loc = this.context.getUniformLocation(program, u_name);
				this.context.uniform4f(loc, x, y, z, w);
			},

			set_uniform_mat4_raw: (program_id, name_ptr, name_len, mat) => {
				let program = this.programs[program_id-1] || null;

				let mat_view = this.engine.wasm.heap_memory_view_f32(mat, 16);
				let u_name = this.engine.wasm.rust_str_to_js(name_ptr, name_len);

				let loc = this.context.getUniformLocation(program, u_name);
				this.context.uniformMatrix4fv(loc, false, mat_view);
			},
		};
	},
};
