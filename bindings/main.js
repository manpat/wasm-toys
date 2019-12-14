"use strict";

import {input_module} from "/wasm-toys/input.js"
import {gl_module} from "/wasm-toys/gl.js"

import {WasmModule, WasmInstance, merge_objects} from "/wasm-toys/common.js";

class Engine {
	constructor(canvas, modules) {
		this.canvas = canvas;

		this.gl = gl_module;
		this.input = input_module;

		this.wasm = null;
		this.exports = null;
	}

	init_with(wasm_instance) {
		this.wasm = wasm_instance;
		this.exports = wasm_instance.exports;

		this.gl.init(this);
		this.input.init(this);
	}

	get_imports() {
		return merge_objects(
			{
				canvas_width: () => this.canvas.width,
				canvas_height: () => this.canvas.height,
			},

			this.gl.imports(this),
			this.input.imports(this),
		);
	}

	update(time) {
		this.canvas.width = this.canvas.clientWidth;
		this.canvas.height = this.canvas.clientHeight;
		
		let client_width = this.gl.context.drawingBufferWidth;
		let client_height = this.gl.context.drawingBufferHeight;

		this.exports.internal_update_viewport(client_width, client_height);
		this.exports.internal_update(time);
		window.requestAnimationFrame(this.update.bind(this));
	}
}



export async function initialise_engine(engine_module, canvas) {
	let engine = new Engine(canvas);
	let engine_instance = await engine_module.instantiate(engine.get_imports());
	engine.init_with(engine_instance);

	// Delay initialisation
	window.requestAnimationFrame((time) => {
		engine.exports.main();
		engine.update(time);
	});

	return engine;
}
