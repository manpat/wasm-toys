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

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

void main() {
    if (dot(v_position, clip_plane.xyz) - clip_plane.w < 0.0) {
        discard;
    }

    // gl_FragColor = vec4(v_color, 1.0);
    gl_FragColor = vec4(hsv2rgb(v_color), 1.0);
}