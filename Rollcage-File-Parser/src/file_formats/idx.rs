use nom::{IResult, multi::many0, number::complete::le_u32};

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct IdxRecord {
  pub file_offset: u32,
  pub compressed_file_length: u32,
  pub decompressed_file_length: u32,
  pub unused : u32
}

fn record(input: &[u8]) -> IResult<&[u8], IdxRecord> {
  let (input, o) = nom::sequence::tuple((le_u32, le_u32, le_u32, le_u32))(input)?;
  let (file_offset, compressed_file_length, decompressed_file_length, unused) = o;
  Ok(
    (input, IdxRecord{file_offset, compressed_file_length, decompressed_file_length, unused})
  )
}

pub fn parse_records(input: &[u8]) -> IResult<&[u8], Vec<IdxRecord>> {
  let x= many0(record)(input)?.1;
  Ok((input, x))
}