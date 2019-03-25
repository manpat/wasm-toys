precision highp float;

uniform mat4 proj_view;

attribute vec3 position;
attribute vec2 uv;

varying vec2 v_uv;

void main() {
	gl_Position = proj_view * vec4(position, 1.0);
	v_uv = uv;
}

/* @@@ */

precision highp float;

uniform sampler2D tex;

varying vec2 v_uv;

void main() {
	vec4 col = texture2D(tex, v_uv);

	if (col.a < 0.5) { discard; }

	gl_FragColor = vec4(col.rgb, 1.0);
}