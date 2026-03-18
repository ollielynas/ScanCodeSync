
The data is split into two different chunks, the current clock time, which needs to be communicated with the digital image or audio recording device, and the other metadata which can be recorded in a JSON format on device and be uploaded to the server later to be used in post to sort the files. 

The JSON data would look something like this

```json
{
	timestamp: u64,
	device_id: u16,
	location: String,
	device_name: String,
	
}
```

Making this metadata time accurate is easy as the current time can just be written into the JSON. 
## Image

there are several  issued that need to be solved when displaying information on a screen and then reading that info with a camera. 

The main issue lies in the way that the screen renders images. We don't know the refresh rate of the screen, nor do we know the 

**Solution 1 - Basic Barcode**

This relies on a 1d barcode format. using the fact that we have access to colors we can define the start and end of the barcode using red and green. 



```glsl
#version 300 es
precision highp float;
precision highp int;

const int timeLength = 32;
const int deviceIdLength = 16;
const int colorPadding = 2;
const int deviceId = 7432;

uniform vec2 u_resolution;
uniform float u_time;
uniform vec2 u_mouse;

out vec4 fragColor;

void main() {
    vec2 uv = gl_FragCoord.xy / u_resolution.xy;
    int x = int(floor(uv.x * float(timeLength + deviceIdLength + colorPadding)));
    int time_ms = int(floor(u_time * 1000.0));
    vec3 col = vec3(0.0, 0.0, 0.0);
    if (x == 0) {
        col = vec3(1.0, 0.0, 0.0);
    }
    if (x >= 1 && x <= timeLength) {
        if (((time_ms >> (x - 1)) & 1) == 1) {
            col = vec3(1.000, 1.000, 1.000);
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
```

that then gives you a barcode that looks something like this:
![[Pasted image 20260317152015.png]]

On this example the time is encoded using 32 bits but in reality 64 bits will be needed to encode the EPOCH time. 

With the current time encoded on the left and the device id on the right. If the shutter speed of the camera is slow the more precise codes become a gray color. This will need to be interpreted differently by the [[Metadata reader & File Sorting]] software. perhaps if the format is in a video the changes of the shade of gray between frames can be used to get a more accurate measure of the time
# Audio

In order to get accurate timecodes with audio, an audio file should be generated for a time in the very near future (like 10-20ms, depending on how long it is likely going to take). 
The data will be encoded using Frequency-Shift Keying (FSK). The frequent switches between 1200 Hz and 2200 Hz, representing 0 and 1. The transmission should start with the fixed pattern: `10110111000` ([Barker Code](https://en.wikipedia.org/wiki/Barker_code)) to represent the start of a transmission.
This is followed by 64 bits representing the EPOCH in ms. This is then followed by the u16 device id. This is then followed by a 64 bit Reed-Solomon error correcting code. 
