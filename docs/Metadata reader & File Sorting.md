
All files recorded in the shoot are dumped into intake folder.

The program starts by reading all of the JSON metadata files. From this it will determine what device ids exist. One of the devices should be set as the master clock.

The video and audio files will be grouped based on the device name in the metadata. They should then be sorted based on the internal timestamp. Each video and audio file is scrubbed for [Time Accurate Metadata](Time%20Accurate%20Metadata.md) and then the offset mappings are recorded into a database.


This is implemented in [Rust Desktop Implementation](Rust%20Desktop%20Implementation.md)
