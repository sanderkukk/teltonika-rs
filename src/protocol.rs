#[derive(Debug, Clone, PartialEq)]
pub struct TeltonikaCodec8 {
    pub data_length: u32,
    pub codec_id: u8,
    pub number_of_data: u8,
    pub avl_data: Vec<AVLData>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AVLData {
    pub timestamp: u64,
    pub priority: u8,
    pub gps: GPSElement,
    pub io: IoElement,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GPSElement {
    pub longitude: f64,
    pub latitude: f64,
    pub altitude: u16,
    pub angle: u16,
    pub visible_satellites: u8,
    pub speed: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IoElementValue {
    pub id: u8,
    pub value: u64,
}
#[derive(Debug, Clone, PartialEq)]
pub struct IoElement {
    pub event_io_id: u8,
    pub number_of_total_io: u8,
    pub number_of_1_byte_elements: u8,
    pub io_1_byte_elements: Option<Vec<IoElementValue>>,
    pub number_of_2_byte_elements: u8,
    pub io_2_byte_elements: Option<Vec<IoElementValue>>,
    pub number_of_4_byte_elements: u8,
    pub io_4_byte_elements: Option<Vec<IoElementValue>>,
    pub number_of_8_byte_elements: u8,
    pub io_8_byte_elements: Option<Vec<IoElementValue>>,
}
