extern crate nom;
use core::str;
use nom::{
    bytes::complete::tag,
    character::{complete::digit1},
    combinator::{ map_res},
    IResult,
};
use std::{str::Utf8Error};

fn str_from_bytes(input: &[u8]) -> Result<&str, Utf8Error> {
    let value = str::from_utf8(input)?;
    Ok(value)
}

fn parse_imei(input: &[u8]) -> IResult<&[u8], &str> {
    map_res(digit1, str_from_bytes)(input)
}

fn get_imei(input: &[u8]) -> IResult<&[u8], &str> {
    let (input, _) = tag(&[0, 15])(input)?;
    let (input, parsed_imei) = parse_imei(input)?;

    Ok((input, parsed_imei))
}

#[test]
fn test_imei_parse() {
    let sample_imei = [
        0, 15, 51, 53, 54, 51, 48, 55, 48, 52, 50, 52, 52, 49, 48, 49, 51,
    ];
    assert_eq!(get_imei(&sample_imei).unwrap().1, "356307042441013")
}
