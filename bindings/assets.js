"use strict";

var engine_internal = engine_internal || {};

engine_internal.asset_module = {
	gl_module: null,

	sprites: {},

	init: function(gl_module) {
		this.gl_module = gl_module;
	},


	imports: function() {
		return {};
	},


	get_sprite_id: function(key) {
		return this.sprites[key] || 0;
	},


	exports: function(exps) {
		return {
			create_sprite: (name, texture,  x, y, w, h) => {
				x = x || 0;
				y = y || 0;
				w = w || 0;
				h = h || 0;
				texture = texture || name;

				if (typeof name !== "string") {
					throw "first argument to create_sprite must be a string";
				}

				let tex_id = this.gl_module.get_named_texture(texture);
				if (tex_id === 0) {
					throw `trying to create sprite with unregistered texture ${texture}`;
				}

				if (Object.prototype.hasOwnProperty.call(this.sprites, name)) {
					throw `sprite with name '${name}' already exists`;
				}

				let id = exps.internal_register_sprite(tex_id, x, y, w, h);
				this.sprites[name] = id;
			},

			create_animated_sprite: (name, texture, frame_length, frames) => {
				if (typeof name !== "string") {
					throw "first argument to create_sprite must be a string";
				}

				if (frames.length % 4 > 0) {
					throw ""; // TODO
				}

				let tex_id = this.gl_module.get_named_texture(texture);
				if (tex_id === 0) {
					throw `trying to create sprite with unregistered texture ${texture}`;
				}

				if (Object.prototype.hasOwnProperty.call(this.sprites, name)) {
					throw `sprite with name '${name}' already exists`;
				}

				let frames_ptr = exps.internal_allocate_i32_vec(frames.length);
				heap_memory_view_i32(frames_ptr, frames.length).set(frames);

				let id = exps.internal_register_animated_sprite(tex_id, frame_length, frames.length/4, frames_ptr);
				this.sprites[name] = id;
			},
		};
	},
};
