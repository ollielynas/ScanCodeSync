#version 300 es
precision highp float;
precision highp int;
const int timeLength = 64;
const int deviceIdLength = 16;
const int colorPadding = 2;
const int deviceId = 7432;
uniform vec2 u_resolution;
uniform int u_time_ms_lo;  // lower 32 bits of ms timestamp
uniform int u_time_ms_hi;  // upper 32 bits of ms timestamp
uniform vec2 u_mouse;
out vec4 fragColor;
void main() {
    vec2 uv = gl_FragCoord.xy / u_resolution.xy;
    int x = int(floor(uv.x * float(timeLength + deviceIdLength + colorPadding)));
    vec3 col = vec3(0.0, 0.0, 0.0);
    if (x == 0) {
        col = vec3(1.0, 0.0, 0.0);
    }
    if (x >= 1 && x <= 32) {
        if (((u_time_ms_lo >> (x - 1)) & 1) == 1) {
            col = vec3(1.0, 1.0, 1.0);
        }
    }
    if (x >= 33 && x <= timeLength) {
        if (((u_time_ms_hi >> (x - 33)) & 1) == 1) {
            col = vec3(1.0, 1.0, 1.0);
        }
    }
    if (x >= timeLength + 1 && x <= timeLength + deviceIdLength) {
        if (((deviceId >> (x - timeLength - 1)) & 1) == 1) {
            col = vec3(1.0, 1.0, 1.0);
        }
    }
    if (x == timeLength + deviceIdLength + colorPadding - 1) {
        col = vec3(0.0, 1.0, 0.0);
    }
    fragColor = vec4(col, 1.0);
}
