precision highp float;

uniform mat4 proj_view;
uniform float particle_scale;

attribute vec3 position;
attribute vec3 color;

varying vec3 v_position;
varying vec3 v_color;

void main() {
    gl_Position = proj_view * vec4(position, 1.0);
	gl_PointSize = particle_scale;

    v_position = position;
    v_color = color;
}


/* @@@ */

precision highp float;

varying vec3 v_position;
varying vec3 v_color;

void main() {
	float dist = 1.0 - 2.0 * length(gl_PointCoord - vec2(0.5));

	// float shape = dist
	// float a = clamp(dist, 0.0, 1.0);
    // gl_FragColor = vec4(v_color, 1.0) * a;

	if (dist < 0.0) {
		discard;
	}

    gl_FragColor = vec4(v_color, 1.0);
}