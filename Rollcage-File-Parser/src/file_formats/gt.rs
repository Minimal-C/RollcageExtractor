use std::convert::TryInto;

use nom::{IResult, bytes::complete::tag, number::complete::le_u32};

const GT_MAGIC: &[u8; 4] = &[0x47, 0x54, 0x32, 0x30]; // "GT20"

pub struct GTHeader {
  pub gt_signature: [u8;4],
  pub gt_uncompressed_size: u32,
  pub gt_overlap: u32, // Overlap for in-situ decompression
  pub gt_skip: u32 // Number of bytes to skip to GT data
}

pub fn parse_magic(input: &[u8]) -> IResult<&[u8], &[u8;4]> {
  let (input, signature) = tag(GT_MAGIC)(input)?;
  Ok((input, signature.try_into().unwrap()))
}

pub fn parse_header(input: &[u8]) -> IResult<&[u8], GTHeader> {
  let (input, signature) = parse_magic(input)?;
  let (input, gt_uncompressed_size) = le_u32(input)?;
  let (input, gt_overlap) = le_u32(input)?;
  let (input, gt_skip) = le_u32(input)?;

  let gt_signature = signature.to_owned();
  Ok(
    (input, 
    GTHeader{
      gt_signature, gt_uncompressed_size, gt_overlap, gt_skip
    })
  )
}

macro_rules! READINFOBIT {
  ($input:ident, $info_bits:ident, $input_index:ident, $info_count:ident) => {
    {
      $info_bits = u32::from_le_bytes($input.get($input_index .. ($input_index+std::mem::size_of::<u32>())).ok_or(DecompressionError::CorruptFile)?.try_into().unwrap());
      $info_count = 32;
      $input_index += std::mem::size_of::<u32>();
    }
};
}

macro_rules! NEXTINFOBIT {
  ($input:ident, $info_bits:ident, $input_index:ident, $info_count:ident) => {
    {
      $info_bits >>= 1;
      $info_count -= 1;
      if $info_count == 0 {
        READINFOBIT!($input, $info_bits, $input_index, $info_count);
      }
    }
};
}

macro_rules! COPY_ARRAY_VALUE {
  ($dest_array:ident, $dest_index:ident, $src_array:ident, $src_index:ident) => {
    {
      *$dest_array.get_mut($dest_index).ok_or(DecompressionError::CorruptFile)? = *$src_array.get($src_index).ok_or(DecompressionError::CorruptFile)?;
    }
};
}

#[derive(Debug)]
pub enum DecompressionError {
    IncorrectFileSignature,
    CorruptFile
}

pub fn decompress(input: &[u8], uncompressed_size: u32) -> Result<Vec<u8>, DecompressionError> {
  match parse_magic(input) {
    Ok(_) => {},
    Err(_) => {return Err(DecompressionError::IncorrectFileSignature)},
}

  let mut output: Vec<u8> = vec![0;uncompressed_size as usize];
  let mut info_bits: u32;
  let mut info_count: u16;

  let mut copy_index: usize;
  let mut copy_count: u16;

  let mut input_index: usize = 0;
  let mut output_index: usize = 0;

  // Advance Past Header
  input_index += std::mem::size_of::<GTHeader>();

  READINFOBIT!(input, info_bits, input_index, info_count);

  loop {
    let lsb = info_bits & 1;
    if lsb == 0{
      COPY_ARRAY_VALUE!(output, output_index, input, input_index);
      input_index += 1;
      output_index +=1;
    } 
    else {
      NEXTINFOBIT!(input, info_bits, input_index, info_count);

      let tmp = info_bits & 1;
      if tmp != 0 {
        copy_count = u16::from_le_bytes(input.get(input_index .. (input_index+std::mem::size_of::<u16>())).ok_or(DecompressionError::CorruptFile)?.try_into().unwrap());
        input_index += std::mem::size_of::<u16>();

        let offset = (copy_count >> 3) as u32 | 0xffffe000;
        copy_index = (output_index as u32).wrapping_add(offset) as usize;

      copy_count &= 7;
      if copy_count != 0 {
          copy_count += 2;
      }
      else {
        copy_count = *input.get(input_index).ok_or(DecompressionError::CorruptFile)? as u16;
        input_index += 1;

        if (copy_count & 128) != 0 {
          copy_index = copy_index.wrapping_sub(0x2000);
        }

        copy_count &= 127;

        if copy_count == 1 {
          break;
        }
        if copy_count == 0 {
          copy_count = u16::from_le_bytes(input.get(input_index .. (input_index+std::mem::size_of::<u16>())).ok_or(DecompressionError::CorruptFile)?.try_into().unwrap());
          input_index += std::mem::size_of::<u16>();
        }
        else {
          copy_count += 2;
        }
      }
      
      while copy_count!=0 {
        COPY_ARRAY_VALUE!(output, output_index, output, copy_index);
        output_index += 1;
        copy_index += 1;
        
        copy_count -= 1;
      }
    }
    else {
      let tmp = (*input.get(input_index).ok_or(DecompressionError::CorruptFile)? as u32).wrapping_sub(256);

      copy_index = ((output_index as u32).wrapping_add(tmp)) as usize;
      input_index += 1;

      COPY_ARRAY_VALUE!(output, output_index, output, copy_index);
      output_index += 1;
      copy_index += 1;

      COPY_ARRAY_VALUE!(output, output_index, output, copy_index);
      output_index += 1;
      copy_index += 1;

      NEXTINFOBIT!(input, info_bits, input_index, info_count);

      if (info_bits & 1) != 0 {
        COPY_ARRAY_VALUE!(output, output_index, output, copy_index);
        output_index += 1;
        copy_index += 1;
        
        COPY_ARRAY_VALUE!(output, output_index, output, copy_index);
        output_index += 1;
        copy_index += 1;
      }
      
      NEXTINFOBIT!(input, info_bits, input_index, info_count);

      if (info_bits & 1) != 0 {
        COPY_ARRAY_VALUE!(output, output_index, output, copy_index);
        output_index += 1;
      }
    }
  }

    NEXTINFOBIT!(input, info_bits, input_index, info_count);
  }

  Ok(output)
}