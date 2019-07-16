"use strict";

var engine_internal = engine_internal || {};

engine_internal.gl_module = {
	programs: [null],
	shaders: [null],
	buffers: [null],

	framebuffers: [null],
	textures: [null],
	named_textures: {},


	init: function(canvas) {
		this.context = this.create_context(canvas, { stencil: true });
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


	imports: function() {
		let gl = this.context;

		return {
			// General state stuff
			viewport: (x,y,w,h) => gl.viewport(x, y, w, h),
			scissor: (x,y,w,h) => gl.scissor(x, y, w, h),

			clear_color: (r,g,b,a) => gl.clearColor(r,g,b,a),
			clear: (x) => gl.clear(x),

			enable: (e) => gl.enable(e),
			disable: (e) => gl.disable(e),

			blend_func: (s, d) => gl.blendFunc(s, d),
			
			// Draw stuff
			draw_arrays: (draw_mode, start, count) => gl.drawArrays(draw_mode, start, count),
			draw_elements: (draw_mode, count, type, offset) => gl.drawElements(draw_mode, count, type, offset),


			// Buffer stuff
			create_buffer: () => {
				this.buffers.push(gl.createBuffer());
				return this.buffers.length;
			},

			bind_buffer: (target, id) => {
				let buf = this.buffers[id-1] || null;
				gl.bindBuffer(target, buf);
			},

			upload_buffer_data: (target, ptr, len) => {
				let buf = heap_memory_view(ptr, len);
				gl.bufferData(target, buf, gl.STATIC_DRAW);
			},

			vertex_attrib_pointer: function (attrib, components, component_type, normalize, stride, offset) {
				gl.vertexAttribPointer(attrib, components, component_type, normalize, stride, offset);
			},

			enable_attribute: (attrib) => gl.enableVertexAttribArray(attrib),
			disable_attribute: (attrib) => gl.disableVertexAttribArray(attrib),


			// Texture stuff
			create_texture: () => {
				this.textures.push(gl.createTexture());
				return this.textures.length;
			},

			bind_texture: (id) => {
				let texture = this.textures[id-1] || null;
				gl.bindTexture(gl.TEXTURE_2D, texture);
			},

			active_texture: (id) => gl.activeTexture(gl.TEXTURE0 + id),

			upload_image_data: (w, h, format, type, ptr, len) => {
				let buf = heap_memory_view(ptr, len);
				gl.texImage2D(gl.TEXTURE_2D, 0, format, w, h, 0,
					format, type, buf);
			},

			tex_parameter: (param, value) => {
				gl.texParameteri(gl.TEXTURE_2D, param, value);
			},


			// Framebuffer stuff
			create_framebuffer: () => {
				this.framebuffers.push(gl.createFramebuffer());
				return this.framebuffers.length;
			},


			// Shader stuff
			create_shader_program: () => {
				this.programs.push(gl.createProgram());
				return this.programs.length;
			},

			create_shader: (type, src_ptr, src_len) => {
				let sh;
				switch (type) {
					case 0: sh = gl.createShader(gl.VERTEX_SHADER); break; 
					case 1: sh = gl.createShader(gl.FRAGMENT_SHADER); break; 
				}

				gl.shaderSource(sh, rust_str_to_js(src_ptr, src_len));
				gl.compileShader(sh);

				let info = gl.getShaderInfoLog(sh);
				if (info.length > 0) {
					console.error(info.slice(0, -1));
				}

				this.shaders.push(sh);
				return this.shaders.length;
			},

			bind_attrib_location: (program_id, name_ptr, name_len, idx) => {
				let program = this.programs[program_id-1] || null;
				let name = rust_str_to_js(name_ptr, name_len);
				gl.bindAttribLocation(program, idx, name);
			},

			link_program: (program_id, vert_id, frag_id) => {
				let program = this.programs[program_id-1] || null;
				let vert = this.shaders[vert_id-1] || null;
				let frag = this.shaders[frag_id-1] || null;

				gl.attachShader(program, vert);
				gl.attachShader(program, frag);
				gl.linkProgram(program);

				let info = gl.getProgramInfoLog(program);
				if (info.length > 0) {
					console.error(info.slice(0, -1));
				}
			},

			use_program: (program_id) => {
				let program = this.programs[program_id-1] || null;
				gl.useProgram(program);
			},

			stencil_func: (condition, reference, mask) => gl.stencilFunc(condition, reference, mask),
			stencil_op: (stencil_fail, depth_fail, pass) => gl.stencilOp(stencil_fail, depth_fail, pass),
			color_mask: (r,g,b,a) => gl.colorMask(r,g,b,a),
			depth_mask: (enabled) => gl.depthMask(enabled),
			stencil_mask: (bits) => gl.stencilMask(bits),

			set_uniform_int_raw: (program_id, name_ptr, name_len, i) => {
				let program = this.programs[program_id-1] || null;
				let u_name = rust_str_to_js(name_ptr, name_len);

				let loc = gl.getUniformLocation(program, u_name);
				gl.uniform1i(loc, i);
			},

			set_uniform_f32_raw: (program_id, name_ptr, name_len, f) => {
				let program = this.programs[program_id-1] || null;
				let u_name = rust_str_to_js(name_ptr, name_len);

				let loc = gl.getUniformLocation(program, u_name);
				gl.uniform1f(loc, f);
			},

			set_uniform_vec4_raw: (program_id, name_ptr, name_len, x,y,z,w) => {
				let program = this.programs[program_id-1] || null;
				let u_name = rust_str_to_js(name_ptr, name_len);

				let loc = gl.getUniformLocation(program, u_name);
				gl.uniform4f(loc, x, y, z, w);
			},

			set_uniform_mat4_raw: (program_id, name_ptr, name_len, mat) => {
				let program = this.programs[program_id-1] || null;
				let buf_raw = engine_internal.memory.buffer;

				let mat_view = new Float32Array(buf_raw, mat, 16);
				let u_name = rust_str_to_js(name_ptr, name_len);

				let loc = gl.getUniformLocation(program, u_name);
				gl.uniformMatrix4fv(loc, false, mat_view);
			},
		};
	},


	exports: function(exps) {
		return {
			register_texture: (id, el) => {
				if (this.named_textures.hasOwnProperty(id)) {
					return;
				}

				switch (typeof el) {
					case "undefined":
						el = document.getElementById(id);
						break;
					case "string":
						el = document.getElementById(el);
						break;
					case "object": break;
					default:
						el = null;
				}

				if (!el) {
					throw `trying to register texture ${id} with invalid type`;	
				}

				let tex_id = this.load_named_texture(id, el);

				// TODO: Make sure this makes sense
				const has_dimensions = el instanceof HTMLImageElement
					|| el instanceof HTMLCanvasElement
					|| el instanceof Image;

				if (has_dimensions) {
					exps.internal_register_texture(tex_id, el.width, el.height);
				}
			},
		};
	},
};
