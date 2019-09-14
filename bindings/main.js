"use strict";

var engine_internal = engine_internal || {};

const merge_objects = Object.assign || function(t) {
    for (var s, i = 1, n = arguments.length; i < n; i++) {
        s = arguments[i];
        for (var p in s) if (Object.prototype.hasOwnProperty.call(s, p))
            t[p] = s[p];
    }
    return t;
};

function create_enum(arr) {
	let new_enum = {};
	for (let i = 0; i < arr.length; i++) {
		new_enum[arr[i]] = i;
	}

	new_enum.length = arr.length;

	new_enum.contains = function(i) {
		return i >= 0 && i < this.length;
	};

	return Object.freeze(new_enum);
}


function heap_memory_view(ptr, len) {
	let buf_raw = engine_internal.memory.buffer;
	return new Uint8Array(buf_raw, ptr, len);
}

function heap_memory_view_i32(ptr, len) {
	let buf_raw = engine_internal.memory.buffer;
	return new Int32Array(buf_raw, ptr, len);
}

function rust_str_to_js(ptr, len) {
	let buf = heap_memory_view(ptr, len);
	return engine_internal.text_decoder.decode(buf);
}

function js_str_to_rust(str) {
	let buf_raw = engine_internal.memory.buffer;
	let src_buf = engine_internal.text_encoder.encode(str);

	let ptr = engine_internal.exports.allocate_str_space(src_buf.length);
	let dst_buf = heap_memory_view(ptr, src_buf.length);
	dst_buf.set(src_buf);
	return ptr;
}


async function initialise_engine(engine_module, canvas, user_imports) {
	if (TextDecoder && TextEncoder) {
		engine_internal.text_encoder = new TextEncoder();
		engine_internal.text_decoder = new TextDecoder();
	} else {
		console.warn("TextEncoder and TextDecoder isn't supported! Using a half-baked hacky solution instead");
		engine_internal.text_encoder = { encode: engine_internal.poormans_text_encode };
		engine_internal.text_decoder = { decode: engine_internal.poormans_text_decode };
	}

	let modules = ["gl", "input"];

	for (let m of modules) {
		let name = m + "_module";
		if (typeof engine_internal[name] !== "object") {
			throw m + ".js is missing";
		}
	}

	engine_internal.canvas = canvas;
	engine_internal.gl_module.init(canvas);
	engine_internal.input_module.init(canvas);

	let wasm_params = {
		// one page = 64kiB
		mem: new WebAssembly.Memory({initial: 10, maximum: 100}),
		env: engine_internal.initialise_imports(user_imports)
	};

	let engine_instance = await WebAssembly.instantiate(engine_module, wasm_params);

	engine_internal.instance = engine_instance;
	engine_internal.memory = engine_instance.exports.memory;
	engine_internal.exports = engine_instance.exports;

	let engine = {};
	engine_internal.initialise_exports(engine);

	let update_fn = function(time) {
		canvas.width = canvas.clientWidth;
		canvas.height = canvas.clientHeight;
		
		let client_width = engine_internal.gl_module.context.drawingBufferWidth;
		let client_height = engine_internal.gl_module.context.drawingBufferHeight;

		engine_internal.exports.internal_update_viewport(client_width, client_height);
		engine_internal.exports.internal_update(time);
		window.requestAnimationFrame(update_fn);
	};

	// Delay initialisation
	window.requestAnimationFrame((time) => {
		engine_internal.exports.main();
		update_fn(time);
	});

	return engine;
}


engine_internal.initialise_imports = function(user_imports) {
	let null_func = function() {};
	let fixed_user_imports = {
		user_init: user_imports.on_init || null_func,
		user_update: user_imports.on_update || null_func,
	};

	let io_imports = {
		console_log_raw: (ptr, len) => {
			console.log(rust_str_to_js(ptr, len));
		},
		console_warn_raw: (ptr, len) => {
			console.warn(rust_str_to_js(ptr, len));
		},
		console_error_raw: (ptr, len) => {
			console.error(rust_str_to_js(ptr, len));
		},
	};

	let util_imports = {
		canvas_width: () => this.canvas.width,
		canvas_height: () => this.canvas.height,
	};

	let math_imports = {
		Math_tan: Math.tan,
		Math_acos: Math.acos,
		Math_asin: Math.asin,
		Math_atan: Math.atan,
		Math_atan2: Math.atan2,
		Math_cbrt: Math.cbrt,
		Math_hypot: Math.hypot,

		math_random: Math.random, 
	};

	return merge_objects({},
		fixed_user_imports,
		
		io_imports,
		util_imports,
		math_imports,

		this.gl_module.imports(),
		this.input_module.imports(),
	);
};


engine_internal.initialise_exports = function(engine) {
	let exps = this.exports;

	merge_objects(
		engine,

		this.input_module.exports(exps),
		this.gl_module.exports(exps),
	);
};


engine_internal.poormans_text_encode = function(str) {
	// Convert to monked utf-8 string
	str = unescape(encodeURIComponent(str));

	let buf = new Uint8Array(str.length);
	for(let i = 0; i < str.length; i++) {
		buf[i] = str.charCodeAt(i) & 0xFF;
	}

	return buf;
};


engine_internal.poormans_text_decode = function(buffer) {
	let string = "";
	for (var i = 0; i < buffer.length; i++) {
		string += String.fromCharCode(buffer[i]);
	}
	return decodeURIComponent(escape(string))
};