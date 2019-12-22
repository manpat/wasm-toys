"use strict";

import {input_module} from "/wasm-toys/input.js"
import {gl_module} from "/wasm-toys/gl.js"


class Engine {
	constructor(wasm_module, canvas) {
		this.canvas = canvas;

		this.gl = gl_module;
		this.input = input_module;

		this.wasm_module = wasm_module;
		this.wasm = null;
		this.exports = null;
		this.imports = null;

		this.workers = [];
	}

	init_with(wasm_instance) {
		this.wasm = wasm_instance;
		this.exports = wasm_instance.exports;

		this.gl.init(this);
		this.input.init(this);
	}

	get_imports() {
		if (this.imports !== null) {
			return this.imports;
		}

		this.imports = {
			canvas_width: () => this.canvas.width,
			canvas_height: () => this.canvas.height,

			fork: (num_threads) => this.fork(num_threads),
		};

		let gl_imports = this.gl.imports(this);
		let input_imports = this.input.imports(this);

		for (let [k, v] of Object.entries(gl_imports)) {
			this.imports[k] = v;
		}

		for (let [k, v] of Object.entries(input_imports)) {
			this.imports[k] = v;
		}

		return this.imports;
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

	fork(num_threads) {
		for (let i = 0; i < num_threads; i++) {
			let worker_id = this.workers.length;
			this.workers.push(null);

			console.log("init worker", worker_id);
			let worker = new Worker("/wasm-toys/worker.js");
			worker.onmessage = this.on_worker_message.bind(this, worker_id, worker);

			// clone module data so it can be moved
			let mod_data = this.wasm_module.raw_data.slice(0);

			let import_data = Object.keys(this.get_imports());

			worker.postMessage(["init", mod_data, import_data], [mod_data]);
		}
	}

	on_worker_message(worker_id, worker, evt) {
		let msg_type = evt.data[0];

		if (msg_type === "data") {
			let buf = this.wasm.js_buf_to_rust(evt.data[1]);
			this.exports.internal_handle_worker_message(worker_id, buf);

		} else if (msg_type === "init_complete") {
			this.workers[worker_id] = worker;
			this.exports.internal_handle_worker_ready(worker_id);
		}
	}
}



export async function initialise_engine(engine_module, canvas) {
	let engine = new Engine(engine_module, canvas);
	let imports = engine.get_imports();
	let engine_instance = await engine_module.instantiate(imports);
	engine.init_with(engine_instance);

	// Delay initialisation
	window.requestAnimationFrame((time) => {
		engine.exports.main();
		engine.update(time);
	});

	return engine;
}
