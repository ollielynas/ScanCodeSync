use anyhow;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DeviceID {
    ClientID(u16),
    CameraID(String),
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DeviceTime {
    internal_clock: u64,
    device_id: DeviceID,
}

/// these data entries should be ripped from footage and metadata files as they are moved into the "processed but not sorted" phase
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DataEntry {
    time: DeviceTime,
    val: DataValue,
}
/// docs\Data Entry Options.md
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DataValue {
    IsMaster(bool),
    IsDirector(bool),
    IsOperator(bool),

    ProductionName(String),

    EnableOperatorName(bool),
    OperatorName(String),

    EnableSceneName(bool),
    SceneName(String),

    EnableTakeNumber(bool),
    TakeNumber(bool),
    // the above values are recorded in a csv format and can be read form either a qr code or a text file

    /// the below value if read from a image of a barcode.
    ClockOffset(DeviceTime)
}

impl DataEntry {
    fn from_csv_row(row: String) -> anyhow::Result<DataEntry> {
        let mut vals: Vec<&str> = row.split(",").collect();
        let cols: [&str; 4] = vals.as_mut_slice().try_into()?;

        return Ok(
            DataEntry { time: DeviceTime {
                internal_clock: cols[1].parse()?,
                device_id: DeviceID::ClientID(cols[0].parse()?),
            }, val: match (cols[2], cols[3]) {

                ("isMaster", "true"|"True") => DataValue::IsMaster(true),
                ("isMaster", "false"|"False") => DataValue::IsMaster(true),

                ("isDirector", "true"|"True") => DataValue::IsDirector(true),
                ("isDirector", "false"|"False") => DataValue::IsDirector(true),

                ("isOperator", "true"|"True") => DataValue::IsOperator(true),
                ("isOperator", "false"|"False") => DataValue::IsOperator(true),

                ("productionName", s) => DataValue::ProductionName(s.to_string()),

                ("enableOperatorName", "true"|"True") => DataValue::EnableOperatorName(true),
                ("enableOperatorName", "false"|"False") => DataValue::EnableOperatorName(false),
                ("operatorName", s) => DataValue::OperatorName(s.to_string()),

                ("enableSceneName", "true"|"True") => DataValue::EnableSceneName(true),
                ("enableSceneName", "false"|"False") => DataValue::EnableSceneName(false),
                ("sceneName", s) => DataValue::SceneName(s.to_string()),

                ("enableTakeNumber", "true"|"True") => DataValue::EnableTakeNumber(true),
                ("enableTakeNumber", "false"|"False") => DataValue::EnableTakeNumber(false),
                ("takeNumber", s) => DataValue::TakeNumber(s.parse()?),


                (k,v) => {anyhow::bail!("failed to parse key value pair; {k}:{v}")}
            }
            }
        )
    }
}
