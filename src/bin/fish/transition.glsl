precision highp float;

uniform float aspect;

attribute vec3 position;
varying vec2 v_uv;

void main() {
    gl_Position = vec4(position, 1.0);

    v_uv = position.xy * vec2(aspect, 1.0);
}


/* @@@ */

precision highp float;

uniform vec4 fade_color;
uniform float fade_amount;

varying vec2 v_uv;

void main() {
	float dist = length(v_uv);

	if (dist < (1.0 - fade_amount)) {
		discard;
	}

    gl_FragColor = fade_color;
}