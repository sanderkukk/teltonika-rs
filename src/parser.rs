extern crate nom;
use core::str;
use crc::{Crc, CRC_16_ARC};
use nom::{
    bytes::complete::{tag, take},
    character::complete::digit1,
    combinator::{map_res, peek},
    error::ErrorKind,
    multi::count,
    number::complete::{be_u16, be_u32, be_u64, be_u8},
    Err, IResult,
};
use std::usize;

use crate::protocol::{AVLData, GPSElement, IoElement, IoElementValue, TeltonikaCodec8};

pub const X25: Crc<u16> = Crc::<u16>::new(&CRC_16_ARC);

fn parse_imei(input: &[u8]) -> IResult<&[u8], &str> {
    map_res(digit1, str::from_utf8)(input)
}

pub fn get_imei(input: &[u8]) -> IResult<&[u8], &str> {
    let (input, _) = tag(&[0, 15])(input)?;
    let (input, parsed_imei) = parse_imei(input)?;

    Ok((input, parsed_imei))
}

fn calculate_crc(avl_data: &[u8]) -> u16 {
    X25.checksum(avl_data)
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

    println!("calculated {:?}", calculated_crc);
    println!("crc : {:?}", crc);

    if calculated_crc as u32 != crc {}

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
    assert_eq!(get_imei(&sample_imei).unwrap().1, "356307042441013")
}

#[test]
fn test_data_parse() {
    // let input = "0000000000000036080400000113fc208dff000f14f650209cca80006f00d60400040004030101150316030001460000015d0000000113fc17610b000f14ffe0209cc580006e00c00500010004030101150316010001460000015e0000000113fc284945000f150f00209cd200009501080400000004030101150016030001460000015d0000000113fc267c5b000f150a50209cccc0009300680400000004030101150016030001460000015b00040000C7CF";
    // let input2 = "000000000000002808010000016B40D9AD80010000000000000000000000000000000103021503010101425E100000010000F22A";
    let input3 = "000000000000003608010000016B40D8EA30010000000000000000000000000000000105021503010101425E0F01F10000601A014E0000000000000000010000C7CF";
    let decoded = hex::decode(input3).expect("Decoding failed");

    println!("decoded{:?}", parse_teltonika_codec_8(&decoded));
}
