"use strict";

var engine_internal = engine_internal || {};

// Should match KeyCode in input.rs
const KeyCode = create_enum([
	"Left", "Right", "Up", "Down",
	"W", "A", "S", "D",
	"Q", "E", "F",

	"Space", "Enter", "Escape",
	"Shift", "Ctrl", "Alt",
	"F1",
]);

// Should match MouseButton in input.rs
const MouseButton = create_enum([
	"Left", "Middle", "Right"
]);

// Should match Intent in input.rs
const Intent = create_enum([
	"Up",
	"Down",
	"Left",
	"Right",

	"Primary",
	"Secondary",
]);


engine_internal.input_module = {
	init: function (canvas) {
		// No context menus pls
		// TODO: forward to engine in editor mode
		document.addEventListener('contextmenu', (e) => e.preventDefault(), true);

		// Defer binding so engine has a chance to init
		window.requestAnimationFrame(() => {
			document.addEventListener('keydown', this.on_key_down.bind(this), true);
			document.addEventListener('keyup', this.on_key_up.bind(this), true);

			document.addEventListener('mousedown', this.on_mouse_down.bind(this), true);
			document.addEventListener('mouseup', this.on_mouse_up.bind(this), true);
			document.addEventListener('mousemove', this.on_mouse_move.bind(this), true);

			// TODO: scroll
			// TODO: touch
			// document.addEventListener('touchstart', this.ontouchstart);
			// document.addEventListener('touchmove', this.ontouchmove);
			// document.addEventListener('touchend', this.ontouchend);
		});

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


	register_editor_view: function(el, id) {
		el.tabIndex = 1; // make sure elements can recieve focus

		el.addEventListener('keydown', this.on_key_down.bind(this), true);
		el.addEventListener('keyup', this.on_key_up.bind(this), true);

		el.addEventListener('mousedown', this.on_mouse_down.bind(this), true);
		el.addEventListener('mouseup', this.on_mouse_up.bind(this), true);
		el.addEventListener('mousemove', this.on_mouse_move.bind(this), true);

		el.addEventListener('focus', this.on_focus_gain.bind(this), true);
		el.addEventListener('blur', this.on_focus_loss.bind(this), true);
	},


	on_focus_loss: function(e) {
		engine_internal.exports.internal_handle_focus_loss();
	},

	on_focus_gain: function(e) {
		engine_internal.exports.internal_handle_focus_gain();
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

