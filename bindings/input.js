"use strict";

var engine_internal = engine_internal || {};

engine_internal.input_module = {
	init: function (canvas) {
		this.canvas = canvas;

		this.has_pointer_lock = 'pointerLockElement' in document ||
			'mozPointerLockElement' in document ||
			'webkitPointerLockElement' in document;
		
		this.request_pointer_lock = canvas.requestPointerLock
			|| canvas.mozRequestPointerLock
			|| canvas.webkitRequestPointerLock
			|| (() => console.error("Pointer lock not supported"));

		this.exit_pointer_lock = document.exitPointerLock
			|| document.mozExitPointerLock
			|| document.webkitExitPointerLock
			|| (() => console.error("Pointer lock not supported"));

		this.request_pointer_lock = this.request_pointer_lock.bind(canvas);
		this.exit_pointer_lock = this.exit_pointer_lock.bind(document);

		if (this.has_pointer_lock) {
			document.addEventListener('pointerlockchange',
				(e) => this.on_pointer_lock_change(e), false);
			canvas.addEventListener('pointerlockerror',
				(e) => this.on_pointer_lock_error(e), false);
		}
	},


	init_input_listeners: function(passive) {
		let target = passive? this.canvas : document;

		if (!passive) {
			// Disable context menus if not in passive mode
			target.addEventListener('contextmenu', (e) => e.preventDefault(), true);
		}

		target.addEventListener('keydown', this.on_key_down.bind(this), true);
		target.addEventListener('keyup', this.on_key_up.bind(this), true);

		target.addEventListener('mousedown', this.on_mouse_down.bind(this), true);
		target.addEventListener('mouseup', this.on_mouse_up.bind(this), true);
		target.addEventListener('mousemove', this.on_mouse_move.bind(this), true);
		target.addEventListener('dblclick', (e) => e.preventDefault(), true);

		// TODO: scroll
		target.addEventListener('touchstart', this.on_touch_down.bind(this), false);
		target.addEventListener('touchmove', this.on_touch_move.bind(this), false);
		target.addEventListener('touchend', this.on_touch_up.bind(this), false);
		target.addEventListener('touchcancel', this.on_touch_up.bind(this), false);

		target.addEventListener('mouseleave', this.on_focus_loss.bind(this), false);
	},


	// on_focus_gain: function(e) {
	// 	engine_internal.exports.internal_handle_focus_gain();
	// },

	on_focus_loss: function(e) {
		engine_internal.exports.internal_handle_focus_loss();
	},


	on_key_down: function(e) {
		let name = js_str_to_rust(e.code);
		let consume = engine_internal.exports.internal_handle_key_down(name);
		if (consume) {
			e.preventDefault();
		}
	},


	on_key_up: function(e) {
		let name = js_str_to_rust(e.code);
		let consume = engine_internal.exports.internal_handle_key_up(name);
		if (consume) {
			e.preventDefault();
		}
	},


	on_mouse_down: function(e) {
		let button = e.button;
		let x = e.clientX;
		let y = e.clientY;
		let consume = engine_internal.exports.internal_handle_mouse_down(button, x, y);
		if (consume) {
			e.preventDefault();
		}
	},


	on_mouse_up: function(e) {
		let button = e.button;
		let x = e.clientX;
		let y = e.clientY;
		let consume = engine_internal.exports.internal_handle_mouse_up(button, x, y);
		if (consume) {
			e.preventDefault();
		}
	},


	on_mouse_move: function(e) {
		let x = e.clientX;
		let y = e.clientY;
		let dx = e.movementX;
		let dy = e.movementY;
		let consume = engine_internal.exports.internal_handle_mouse_move(x, y, dx, dy);
		if (consume) {
			e.preventDefault();
		}
	},


	on_touch_down: function(e) {
		let consume = false;

		for (let touch of e.changedTouches) {
			let x = touch.clientX;
			let y = touch.clientY;
			consume |= engine_internal.exports.internal_handle_touch_down(touch.identifier, x, y);
		}

		if (consume) {
			e.preventDefault();
		}
	},


	on_touch_up: function(e) {
		let consume = false;

		for (let touch of e.changedTouches) {
			let x = touch.clientX;
			let y = touch.clientY;
			consume |= engine_internal.exports.internal_handle_touch_up(touch.identifier, x, y);
		}

		if (consume) {
			e.preventDefault();
		}
	},


	on_touch_move: function(e) {
		let consume = false;

		for (let touch of e.changedTouches) {
			let x = touch.clientX;
			let y = touch.clientY;
			consume |= engine_internal.exports.internal_handle_touch_move(touch.identifier, x, y);
		}

		if (consume) {
			e.preventDefault();
		}
	},


	on_pointer_lock_change: function(e) {
		let lock_element = document.pointerLockElement
			|| document.mozPointerLockElement
			|| document.webkitPointerLockElement;

		let enabled = (lock_element === this.canvas);

		engine_internal.exports.internal_notify_pointer_lock_change(enabled);
	},

	on_pointer_lock_error: function(e) {
		console.error("pointer lock failed");
	},


	imports: function() {
		return {
			init_input_listeners: (passive) => this.init_input_listeners(passive),

			request_pointer_lock: () => this.request_pointer_lock(),
			exit_pointer_lock: () => this.exit_pointer_lock(),
		};
	},

	exports: function(exps) {
		// TODO: Think about naming
		return {
			enable_pointer_lock: exps.engine_enable_pointer_lock,
		};
	},
};

