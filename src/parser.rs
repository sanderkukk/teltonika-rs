extern crate nom;
use crate::protocol::{AVLData, GPSElement, IoElement, IoElementValue, TeltonikaCodec8};
use core::str;
use crc::{Crc, CRC_16_ARC};
use nom::{
    bytes::complete::{tag, take},
    character::complete::digit1,
    combinator::{map_res, peek},
    multi::count,
    number::complete::{be_u16, be_u32, be_u64, be_u8},
    IResult,
};
use std::usize;

fn get_imei(input: &[u8]) -> IResult<&[u8], &str> {
    map_res(digit1, str::from_utf8)(input)
}

pub fn parse_teltonika_imei(input: &[u8]) -> IResult<&[u8], &str> {
    let (input, _) = tag(&[0, 15])(input)?;
    let (input, parsed_imei) = get_imei(input)?;

    Ok((input, parsed_imei))
}

fn calculate_crc(avl_data: &[u8]) -> u16 {
    Crc::<u16>::new(&CRC_16_ARC).checksum(avl_data)
}

fn get_1_byte_io_element(input: &[u8]) -> IResult<&[u8], IoElementValue> {
    let (input, id) = be_u8(input)?;
    let (input, value) = be_u8(input)?;

    Ok((
        input,
        IoElementValue {
            id,
            value: value.into(),
        },
    ))
}
fn get_2_byte_io_element(input: &[u8]) -> IResult<&[u8], IoElementValue> {
    let (input, id) = be_u8(input)?;
    let (input, value) = be_u16(input)?;

    Ok((
        input,
        IoElementValue {
            id,
            value: value.into(),
        },
    ))
}
fn get_4_byte_io_element(input: &[u8]) -> IResult<&[u8], IoElementValue> {
    let (input, id) = be_u8(input)?;
    let (input, value) = be_u32(input)?;

    Ok((
        input,
        IoElementValue {
            id,
            value: value.into(),
        },
    ))
}
fn get_8_byte_io_element(input: &[u8]) -> IResult<&[u8], IoElementValue> {
    let (input, id) = be_u8(input)?;
    let (input, value) = be_u64(input)?;

    Ok((
        input,
        IoElementValue {
            id,
            value: value.into(),
        },
    ))
}

fn get_io_elements(input: &[u8]) -> IResult<&[u8], IoElement> {
    let (input, event_io_id) = be_u8(input)?;
    let (input, number_of_total_io) = be_u8(input)?;

    let (input, number_of_1_byte_elements) = be_u8(input)?;
    let (input, io_1_byte_elements) =
        count(get_1_byte_io_element, number_of_1_byte_elements.into())(input)?;

    let (input, number_of_2_byte_elements) = be_u8(input)?;
    let (input, io_2_byte_elements) =
        count(get_2_byte_io_element, number_of_2_byte_elements.into())(input)?;

    let (input, number_of_4_byte_elements) = be_u8(input)?;
    let (input, io_4_byte_elements) =
        count(get_4_byte_io_element, number_of_4_byte_elements.into())(input)?;

    let (input, number_of_8_byte_elements) = be_u8(input)?;
    let (input, io_8_byte_elements) =
        count(get_8_byte_io_element, number_of_8_byte_elements.into())(input)?;

    Ok((
        input,
        IoElement {
            event_io_id,
            number_of_total_io,
            number_of_1_byte_elements,
            io_1_byte_elements: Some(io_1_byte_elements),
            number_of_2_byte_elements,
            io_2_byte_elements: Some(io_2_byte_elements),
            number_of_4_byte_elements,
            io_4_byte_elements: Some(io_4_byte_elements),
            number_of_8_byte_elements,
            io_8_byte_elements: Some(io_8_byte_elements),
        },
    ))
}

fn get_gps_element(input: &[u8]) -> IResult<&[u8], GPSElement> {
    let (input, raw_longitude) = be_u32(input)?;
    let longitude: f64 = raw_longitude as f64 / 1e7f64;
    let (input, raw_latitude) = be_u32(input)?;
    let latitude: f64 = raw_latitude as f64 / 1e7f64;
    let (input, altitude) = be_u16(input)?;
    let (input, angle) = be_u16(input)?;
    let (input, visible_satellites) = be_u8(input)?;
    let (input, speed) = be_u16(input)?;

    Ok((
        input,
        GPSElement {
            longitude,
            latitude,
            altitude,
            angle,
            visible_satellites,
            speed,
        },
    ))
}

fn get_avl_data(input: &[u8]) -> IResult<&[u8], AVLData> {
    let (input, timestamp) = be_u64(input)?;
    let (input, priority) = be_u8(input)?;
    let (input, gps) = get_gps_element(input)?;
    let (input, io) = get_io_elements(input)?;

    Ok((
        input,
        AVLData {
            timestamp,
            priority,
            gps,
            io,
        },
    ))
}

pub fn parse_teltonika_codec_8(input: &[u8]) -> IResult<&[u8], TeltonikaCodec8> {
    let (input, _) = tag(&[0, 0, 0, 0])(input)?;

    let (input, data_length) = be_u32(input)?;

    let (_, raw_avl_data) = peek(take((data_length) as usize))(input)?;
    let calculated_crc = calculate_crc(raw_avl_data);

    let (input, codec_id) = be_u8(input)?;
    let (input, number_of_data) = be_u8(input)?;
    let (input, avl_data) = count(get_avl_data, number_of_data.into())(input)?;

    let (input, _number_of_data_2) = be_u8(input)?;
    let (input, crc) = be_u32(input)?;

    if calculated_crc as u32 != crc {
        println!("crc no match");
    }

    Ok((
        input,
        TeltonikaCodec8 {
            data_length,
            codec_id,
            number_of_data,
            avl_data,
        },
    ))
}

#[test]
fn test_imei_parse() {
    let sample_imei = [
        0, 15, 51, 53, 54, 51, 48, 55, 48, 52, 50, 52, 52, 49, 48, 49, 51,
    ];
    assert_eq!(
        parse_teltonika_imei(&sample_imei).unwrap().1,
        "356307042441013"
    )
}

#[test]
fn test_data_parse_1() {
    let input3 = "000000000000003608010000016B40D8EA30010000000000000000000000000000000105021503010101425E0F01F10000601A014E0000000000000000010000C7CF";
    let decoded = hex::decode(input3).unwrap();
    let parsed_data = parse_teltonika_codec_8(&decoded).unwrap().1;
    assert_eq!(
        parsed_data,
        TeltonikaCodec8 {
            data_length: 54,
            codec_id: 8,
            number_of_data: 1,
            avl_data: [AVLData {
                timestamp: 1560161086000,
                priority: 1,
                gps: GPSElement {
                    longitude: 0.0,
                    latitude: 0.0,
                    altitude: 0,
                    angle: 0,
                    visible_satellites: 0,
                    speed: 0
                },
                io: IoElement {
                    event_io_id: 1,
                    number_of_total_io: 5,
                    number_of_1_byte_elements: 2,
                    io_1_byte_elements: Some(
                        [
                            IoElementValue { id: 21, value: 3 },
                            IoElementValue { id: 1, value: 1 }
                        ]
                        .to_vec()
                    ),
                    number_of_2_byte_elements: 1,
                    io_2_byte_elements: Some(
                        [IoElementValue {
                            id: 66,
                            value: 24079
                        }]
                        .to_vec()
                    ),
                    number_of_4_byte_elements: 1,
                    io_4_byte_elements: Some(
                        [IoElementValue {
                            id: 241,
                            value: 24602
                        }]
                        .to_vec()
                    ),
                    number_of_8_byte_elements: 1,
                    io_8_byte_elements: Some([IoElementValue { id: 78, value: 0 }].to_vec())
                }
            }]
            .to_vec()
        }
    )
}

#[test]
fn test_data_parse_2() {
    let input3 = "0000000000000036080400000113fc208dff000f14f650209cca80006f00d60400040004030101150316030001460000015d0000000113fc17610b000f14ffe0209cc580006e00c00500010004030101150316010001460000015e0000000113fc284945000f150f00209cd200009501080400000004030101150016030001460000015d0000000113fc267c5b000f150a50209cccc0009300680400000004030101150016030001460000015b00040000F991";
    let decoded = hex::decode(input3).unwrap();
    let parsed_data = parse_teltonika_codec_8(&decoded).unwrap().1;
    assert_eq!(
        parsed_data,
        TeltonikaCodec8 {
            data_length: 54,
            codec_id: 8,
            number_of_data: 4,
            avl_data: [
                AVLData {
                    timestamp: 1185345998335,
                    priority: 0,
                    gps: GPSElement {
                        longitude: 25.3032016,
                        latitude: 54.7146368,
                        altitude: 111,
                        angle: 214,
                        visible_satellites: 4,
                        speed: 4
                    },
                    io: IoElement {
                        event_io_id: 0,
                        number_of_total_io: 4,
                        number_of_1_byte_elements: 3,
                        io_1_byte_elements: Some(
                            [
                                IoElementValue { id: 1, value: 1 },
                                IoElementValue { id: 21, value: 3 },
                                IoElementValue { id: 22, value: 3 }
                            ]
                            .to_vec()
                        ),
                        number_of_2_byte_elements: 0,
                        io_2_byte_elements: Some([].to_vec()),
                        number_of_4_byte_elements: 1,
                        io_4_byte_elements: Some([IoElementValue { id: 70, value: 349 }].to_vec()),
                        number_of_8_byte_elements: 0,
                        io_8_byte_elements: Some([].to_vec())
                    }
                },
                AVLData {
                    timestamp: 1185345397003,
                    priority: 0,
                    gps: GPSElement {
                        longitude: 25.3034464,
                        latitude: 54.7145088,
                        altitude: 110,
                        angle: 192,
                        visible_satellites: 5,
                        speed: 1
                    },
                    io: IoElement {
                        event_io_id: 0,
                        number_of_total_io: 4,
                        number_of_1_byte_elements: 3,
                        io_1_byte_elements: Some(
                            [
                                IoElementValue { id: 1, value: 1 },
                                IoElementValue { id: 21, value: 3 },
                                IoElementValue { id: 22, value: 1 }
                            ]
                            .to_vec()
                        ),
                        number_of_2_byte_elements: 0,
                        io_2_byte_elements: Some([].to_vec()),
                        number_of_4_byte_elements: 1,
                        io_4_byte_elements: Some([IoElementValue { id: 70, value: 350 }].to_vec()),
                        number_of_8_byte_elements: 0,
                        io_8_byte_elements: Some([].to_vec())
                    }
                },
                AVLData {
                    timestamp: 1185346505029,
                    priority: 0,
                    gps: GPSElement {
                        longitude: 25.3038336,
                        latitude: 54.7148288,
                        altitude: 149,
                        angle: 264,
                        visible_satellites: 4,
                        speed: 0
                    },
                    io: IoElement {
                        event_io_id: 0,
                        number_of_total_io: 4,
                        number_of_1_byte_elements: 3,
                        io_1_byte_elements: Some(
                            [
                                IoElementValue { id: 1, value: 1 },
                                IoElementValue { id: 21, value: 0 },
                                IoElementValue { id: 22, value: 3 }
                            ]
                            .to_vec()
                        ),
                        number_of_2_byte_elements: 0,
                        io_2_byte_elements: Some([].to_vec()),
                        number_of_4_byte_elements: 1,
                        io_4_byte_elements: Some([IoElementValue { id: 70, value: 349 }].to_vec()),
                        number_of_8_byte_elements: 0,
                        io_8_byte_elements: Some([].to_vec())
                    }
                },
                AVLData {
                    timestamp: 1185346387035,
                    priority: 0,
                    gps: GPSElement {
                        longitude: 25.3037136,
                        latitude: 54.7146944,
                        altitude: 147,
                        angle: 104,
                        visible_satellites: 4,
                        speed: 0
                    },
                    io: IoElement {
                        event_io_id: 0,
                        number_of_total_io: 4,
                        number_of_1_byte_elements: 3,
                        io_1_byte_elements: Some(
                            [
                                IoElementValue { id: 1, value: 1 },
                                IoElementValue { id: 21, value: 0 },
                                IoElementValue { id: 22, value: 3 }
                            ]
                            .to_vec()
                        ),
                        number_of_2_byte_elements: 0,
                        io_2_byte_elements: Some([].to_vec()),
                        number_of_4_byte_elements: 1,
                        io_4_byte_elements: Some([IoElementValue { id: 70, value: 347 }].to_vec()),
                        number_of_8_byte_elements: 0,
                        io_8_byte_elements: Some([].to_vec())
                    }
                }
            ]
            .to_vec()
        }
    )
}
