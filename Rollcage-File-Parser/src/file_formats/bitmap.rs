use nom::{bytes::complete::tag, IResult};

static BITMAP_MAGIC: &[u8; 2] = &[0x42, 0x4d];

pub fn parse_magic(input: &[u8]) -> IResult<&[u8], ()> {
    let (input, _signature) = tag(BITMAP_MAGIC)(input)?;
    Ok((input, ()))
}