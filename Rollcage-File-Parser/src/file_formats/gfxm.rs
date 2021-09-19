use std::convert::TryInto;

use nom::{IResult, bytes::complete::{tag}, number::complete::le_u32};

const GFXM_MAGIC: &[u8; 4] = &[0x47, 0x46, 0x58, 0x4D]; // "GFXM"

#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
pub struct GfxmHeader {
  pub signature: [u8;GFXM_MAGIC.len()],
  pub unknown_1: u32,
  pub unknown_2: u32,
  pub num_coordinates: u32,
  pub num_segm_sections: u32,
  pub segm_table_offset: u32,
  pub modl_table_offset: u32
}

pub fn parse_magic(input: &[u8]) -> IResult<&[u8], &[u8;GFXM_MAGIC.len()]>{
  let (input, signature) = tag(GFXM_MAGIC)(input)?;
  Ok((input, signature.try_into().unwrap()))
}

pub fn parse_gfxm_header(input: &[u8]) -> IResult<&[u8], GfxmHeader> {
  let (input, signature) = parse_magic(input)?;
  let (input, unknown_1) = le_u32(input)?;
  let (input, unknown_2) = le_u32(input)?;
  let (input, num_coordinates) = le_u32(input)?;
  let (input, num_segm_sections) = le_u32(input)?;
  let (input, segm_table_offset) = le_u32(input)?;
  let (input, modl_table_offset) = le_u32(input)?;

  Ok(
    (input, 
    GfxmHeader{signature: *signature, unknown_1, unknown_2,
       num_coordinates, num_segm_sections, segm_table_offset, modl_table_offset
    })
  )
}