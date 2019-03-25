precision highp float;

uniform mat4 proj_view;

attribute vec3 position;
attribute vec3 color;

varying vec3 v_position;
varying vec3 v_color;

void main() {
	gl_Position = proj_view * vec4(position, 1.0);
	v_position = position;
	v_color = color;
}


/* @@@ */


precision highp float;

uniform vec4 clip_plane;

varying vec3 v_position;
varying vec3 v_color;

void main() {
	if (dot(v_position, clip_plane.xyz) - clip_plane.w < 0.0) {
		discard;
	}

	gl_FragColor = vec4(v_color, 1.0);
}