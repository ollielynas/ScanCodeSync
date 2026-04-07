use crate::data::data_entry::{DeviceTime, TimelineEntry};

pub fn detect_barcode_1d(lst: &[(u8,u8,u8)]) -> Option<(u64, u16)> {
    return None;
}

pub fn detect_barcodes(data: &[u8], width: u32, height: u32, device_time: &DeviceTime) -> anyhow::Result<Vec<TimelineEntry>>  {




    return Ok(vec![]);
}
