"use strict";

export const merge_objects = Object.assign || function(t) {
    for (var s, i = 1, n = arguments.length; i < n; i++) {
        s = arguments[i];
        for (var p in s) if (Object.prototype.hasOwnProperty.call(s, p))
            t[p] = s[p];
    }
    return t;
};


function poormans_text_encode(str) {
	// Convert to monked utf-8 string
	str = unescape(encodeURIComponent(str));

	let buf = new Uint8Array(str.length);
	for(let i = 0; i < str.length; i++) {
		buf[i] = str.charCodeAt(i) & 0xFF;
	}

	return buf;
}

function poormans_text_decode(buffer) {
	let string = "";
	for (var i = 0; i < buffer.length; i++) {
		string += String.fromCharCode(buffer[i]);
	}
	return decodeURIComponent(escape(string))
}

let text_encoder;
let text_decoder;

if (TextDecoder && TextEncoder) {
	text_encoder = new TextEncoder();
	text_decoder = new TextDecoder();
} else {
	console.warn("TextEncoder and TextDecoder isn't supported! Using a half-baked hacky solution instead");
	text_encoder = { encode: poormans_text_encode };
	text_decoder = { decode: poormans_text_decode };
}


export class WasmModule {
	constructor(mod) {
		this.module = mod;
	}

	static async compile(url) {
		let fetch_res = await fetch(url);
		let mod_data = await fetch_res.arrayBuffer();
		let mod = await WebAssembly.compile(mod_data);
		return new WasmModule(mod);
	}

	async instantiate(imports) {
		let box_inst = {inst:null};

		let wasm_params = {
			// one page = 64kiB
			mem: new WebAssembly.Memory({initial: 10, maximum: 100}),
			env: WasmModule.initialise_imports(box_inst, imports)
		};

		let wasm_instance = await WebAssembly.instantiate(this.module, wasm_params);
		let instance = new WasmInstance(wasm_instance);
		box_inst.inst = instance;
		return instance;
	}

	static initialise_imports(box_inst, imports) {
		let io_imports = {
			console_log_raw: (ptr, len) => {
				console.log(box_inst.inst.rust_str_to_js(ptr, len));
			},
			console_warn_raw: (ptr, len) => {
				console.warn(box_inst.inst.rust_str_to_js(ptr, len));
			},
			console_error_raw: (ptr, len) => {
				console.error(box_inst.inst.rust_str_to_js(ptr, len));
			},
		};

		let math_imports = {
			math_random: Math.random, 
		};

		return merge_objects({},
			imports,
			io_imports,
			math_imports,
		);
	}
}



export class WasmInstance {
	constructor(inst) {
		this.instance = inst;
		this.memory = inst.exports.memory;
		this.exports = inst.exports;
	}

	heap_memory_view(ptr, len) {
		if (!ptr) {
			return null;
		}
		
		let buf_raw = this.memory.buffer;
		return new Uint8Array(buf_raw, ptr, len);
	}

	heap_memory_view_u32(ptr, len) {
		if (!ptr) {
			return null;
		}

		let buf_raw = this.memory.buffer;
		return new Uint32Array(buf_raw, ptr, len);
	}

	heap_memory_view_f32(ptr, len) {
		if (!ptr) {
			return null;
		}

		let buf_raw = this.memory.buffer;
		return new Float32Array(buf_raw, ptr, len);
	}

	rust_str_to_js(ptr, len) {
		let buf = this.heap_memory_view(ptr, len);
		return text_decoder.decode(buf);
	}

	js_str_to_rust(str) {
		let buf_raw = this.memory.buffer;
		let src_buf = text_encoder.encode(str);

		let ptr = this.exports.allocate_arena_space(src_buf.length);
		let dst_buf = this.heap_memory_view(ptr, src_buf.length);
		dst_buf.set(src_buf);
		return ptr;
	}


	rust_buf_to_js(ptr, len) {
		let buf = this.heap_memory_view(ptr, len);
		return buf;
	}

	js_buf_to_rust(src_buf) {
		let buf_raw = this.memory.buffer;

		let ptr = this.exports.allocate_arena_space(src_buf.length);
		let dst_buf = this.heap_memory_view(ptr, src_buf.length);
		dst_buf.set(src_buf);
		return ptr;
	}
}