## Scan Code Sync


The purpose  of this standard is to create an accurate timecode and metadata storage system that can be recorded into video and audio mediums. This has the advantage of syncing the internal clocks of many different devices, such as professional and amateur cameras as well as stand alone microphone recorders. The goal is to be able to take a photo or short audio clip from one device onto another (e.g a photo of an android device on a dlsr camera) and then calculate the drift of the clock as well as the offset. These recording became data points of the relative times of the two devices. Using these offsets the total offset of any given device from a master clock can be estimated.

there are 3 components to this solution.

 a.) **[Director Metadata Recorder](Director%20Metadata%20Recorder.md)**, which records things like the current take and the current scene. This data can then be used to sort all of the recording in post. This doesn't need to be formatted as an image or audio clip. It can instead be recorded on device and be later loaded into the recording sorting software. 

b.) **[Operator Metadata Generator](Operator%20Metadata%20Generator.md)**, This needs to be able to generate metadata like name of the operator (e.g wide angle cam 1), as well as the current location data. It also needs to accurately be able to display the state of the internal clock as an image. This image needs to be robust enough to survive the effects of both different refresh rate displays as well as the rolling shutter. It also needs to be able to play a short audio clip through the devices speakers which can accurately be decoded into the metadata as well as an extremely accurate internal clock measurement. This software would be best run on Android/IOS devices

c.) **Master clock** This device is very similar to the Operator metadata generator but it does not need to be able to generate any specific metadata. Its only purpose is to generate a timecode that can be scanned by the operator metadata recorder to keep them all in sync. 

c.) **[Metadata reader & File Sorting](Metadata%20reader%20&%20File%20Sorting.md)**, This software takes in all of the raw files from the cameras and recorders as well as any metadata from the director metadata recorder.
