"use strict";

self.importScripts("/wasm-toys/common.js");

let instance = null;

async function worker_msg(data) {
	let msg_type = data[0];

	if (msg_type === "init") {
		let module_data = data[1];
		let import_names = data[2];

		let imports = {
			send_data: (ptr, len) => {
				let buf = new Uint8Array(instance.rust_buf_to_js(ptr, len));
				self.postMessage(["data", buf], [buf]);
			}
		};

		for (let name of import_names) {
			imports[name] = function () {
				throw `Attempting to call unbound function "${name}" in worker`;
			};
		}

		let worker_module = new WasmModule(await WebAssembly.compile(module_data));
		instance = await worker_module.instantiate(imports);
		instance.exports.worker_main();
		self.postMessage(["init_complete"]);

	} else if (msg_type == "data") {
		if (!instance) {
			throw "Data received on uninitialised worker";
		}

		let data_view = new Uint8Array(data[1]);
		let ptr = instance.js_buf_to_rust(data_view);
		instance.exports.on_message(ptr);
	}
}


self.onmessage = async function(e) {
	await worker_msg(e.data)
		.catch(e => console.error(e));
};