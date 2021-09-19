use std::convert::TryInto;

use image::{ImageBuffer, Rgba, RgbaImage};
use nom::{IResult, bytes::complete::{tag, take}, multi::many_m_n, number::complete::{le_u16, le_u32, le_u8}};

extern crate image;

const BTP_MAGIC: &[u8; 4] = &[0x42, 0x54, 0x50, 0x20]; // "BTP "

#[derive(Debug, Clone, Copy)]
pub struct BtpHeader {
    pub signature: [u8; BTP_MAGIC.len()],
    pub unknown_1: u32,
    pub unknown_2: u32,
    pub unknown_3: u32,
    pub unknown_4: u32,
    pub num_cobjects: u32,
    pub unknown_5: u32,
    pub skybox_data_offset: u32,
    pub unknown_6: u32,
    pub texture_data_offset: u32,
    pub cobjects_data_offset: u32,
    pub unknown_7: u32,
    pub unknown_8: u32,
    pub num_textures: u16,
    pub num_palettes: u16,
    pub texture_page_table_offset: u32,
    pub palette_data_offset: u32,
}

pub fn parse_magic(input: &[u8]) -> IResult<&[u8], &[u8;BTP_MAGIC.len()]> {
  let (input, signature) = tag(BTP_MAGIC)(input)?;
  Ok((input, signature.try_into().unwrap()))
}

pub fn parse_btp_header(input: &[u8]) -> IResult<&[u8], BtpHeader> {
  let (input, signature) = parse_magic(input)?;
  let (input, unknown_1) = le_u32(input)?; //take(16_usize)(input)?;
  let (input, unknown_2) = le_u32(input)?;
  let (input, unknown_3) = le_u32(input)?;
  let (input, unknown_4) = le_u32(input)?;
  let (input, num_cobjects) = le_u32(input)?;
  let (input, unknown_5) = le_u32(input)?;
  let (input, skybox_data_offset) = le_u32(input)?;
  let (input, unknown_6) = le_u32(input)?;
  let (input, texture_data_offset) = le_u32(input)?;
  let (input, cobjects_data_offset) = le_u32(input)?;
  let (input, unknown_7) = le_u32(input)?;
  let (input, unknown_8) = le_u32(input)?;
  let (input, num_textures) = le_u16(input)?;
  let (input, num_palettes) = le_u16(input)?;
  let (input, texture_page_table_offset) = le_u32(input)?;
  let (input, palette_data_offset) = le_u32(input)?;

  Ok(
    (input, 
    BtpHeader{ signature: *signature, unknown_1, unknown_2, unknown_3, unknown_4,
       num_cobjects, unknown_5, skybox_data_offset, unknown_6, texture_data_offset, cobjects_data_offset, unknown_7, unknown_8, num_textures, num_palettes, texture_page_table_offset, palette_data_offset 
    })
  )
}

#[derive(Debug, Clone, Copy)]
pub struct TexturePageInfo {
  pub width: u16,
  pub height: u16,
  pub palette: u32,
  pub texture_offset: u32
}

#[derive(Debug, Clone, Copy)]
pub struct Colour {
  pub red: u8,
  pub green: u8,
  pub blue: u8,
  pub alpha: u8
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
  pub data: [Colour; 256]
}

#[derive(Debug, Clone)]
pub struct Texture {
  pub info: TexturePageInfo,
  pub palette: Palette,
  pub image_data: Vec<u8>
}

impl<'a> Texture {

  pub fn new(info: TexturePageInfo, palette: Palette, image_data: Vec<u8>) -> Result<Self, &'a str> {
    // validation
    match (info.width, info.height) {
      (0,0) => return Err("Texture width and height are zero."),
      (0,_) => return Err("Texture width is zero."),
      (_,0) => return Err("Texture height is zero."),
      (_,_) => {}
    }

    if palette.data.is_empty() {
      return Err("Texture palette data is not present.")
    }

    let is_palette_and_data_valid: bool = image_data.iter().all(|lookup_index| palette.data.get(*lookup_index as usize).is_none());
    if is_palette_and_data_valid {
      Err("Texture data or palette is invalid. Some palette lookups were out of bounds.")
    }
    else {
      Ok(Self{info, palette, image_data})
    }
  }

  pub fn to_rgba_image(&self) -> RgbaImage {
    let width: u32 = self.info.width.into();
    let height: u32 = self.info.height.into();
    
    ImageBuffer::from_fn(width, height, |x,y| {
      let index = (y*width) + x;
      let i = self.image_data[index as usize];
      let pixel_color = self.palette.data.get(i as usize).unwrap();
      Rgba([pixel_color.red, pixel_color.green, pixel_color.blue, pixel_color.alpha])
    })
  }
}

fn parse_texture_page_table( input: &[u8] ) -> IResult<&[u8], TexturePageInfo> {
  let (input, o) = nom::sequence::tuple((le_u16, le_u16, le_u32, le_u32))(input)?;
  let (width, height, palette, texture_offset) = o;
  Ok(
    (input, TexturePageInfo{width, height, palette, texture_offset})
  )
}

pub fn parse_texture_page_infos(input: &[u8], texture_page_table_offset: usize, num_textures: usize ) -> IResult<&[u8], Vec<TexturePageInfo>> {
  let input = input.get(texture_page_table_offset ..).ok_or(nom::Err::Incomplete(nom::Needed::Unknown))?;

  let texture_page_infos= many_m_n(0, num_textures, parse_texture_page_table)(input)?.1;
  Ok((input, texture_page_infos))
}

pub fn parse_palettes(input: &[u8], palette_data_offset: usize, num_palettes: usize) -> IResult<&[u8], Vec<Palette>> {
  let input = input.get(palette_data_offset as usize ..).ok_or(nom::Err::Incomplete(nom::Needed::Unknown))?;
  let palettes= many_m_n(0, num_palettes as usize, parse_palette)(input)?.1;
  Ok((input, palettes))
}

pub fn parse_palette(input: &[u8]) -> IResult<&[u8], Palette> {
  let (input, x) = many_m_n(0, 256, parse_colour)(input)?;
  let data: [Colour; 256] = x.try_into().unwrap();
  Ok((input, Palette{ data }))
}

pub fn parse_colour(input: &[u8]) -> IResult<&[u8], Colour> {
  let (input, (blue, green, red, alpha)) = nom::sequence::tuple((le_u8, le_u8, le_u8, le_u8))(input)?;
  let colour = Colour{red, green, blue, alpha};
  Ok((input, colour))
}

pub fn parse_texture_data<'a>(input: &'a[u8], header: &BtpHeader, texture_page_info: &TexturePageInfo) -> IResult<&'a [u8], Vec<u8>> {
  let start_index: usize = header.texture_data_offset as usize +texture_page_info.texture_offset as usize;
  let num_bytes: usize = texture_page_info.height as usize * texture_page_info.width as usize;
  let input = input.get( start_index ..).ok_or(nom::Err::Incomplete(nom::Needed::Unknown))?;
  let (input, texture_data) = take(num_bytes)(input)?;
  Ok( (input, texture_data.to_vec()) )
}

pub fn parse_textures<'a>(input: &'a [u8], header: &BtpHeader) -> IResult<&'a [u8], Vec<Texture>> {
  let mut textures: Vec<Texture> = Vec::new();
  let texture_infos = parse_texture_page_infos(input,
  header.texture_page_table_offset as usize, header.num_textures as usize).unwrap().1;
  let palettes = parse_palettes(input, header.palette_data_offset as usize, header.num_palettes as usize).unwrap().1;
  
  if header.num_textures == 0 || header.num_palettes == 0 {
      Ok((input, textures))
  }
  else {
    for texture_index in 0..header.num_textures as usize {
      let texture_info= *texture_infos.get(texture_index).unwrap();
      let palette = *palettes.get(texture_info.palette as usize).unwrap();
      if texture_info.width == 0 || texture_info.height == 0 {
          continue;
      }
      let texture_data = parse_texture_data(input, &header, &texture_info).unwrap().1;
      let t = match Texture::new(texture_info, palette, texture_data) {
        Ok(tex) => tex,
        Err(e) => {
          println!("Failed to create texture: {}", e);
          continue;
        }
      };

      textures.push(t);
    }
    
    Ok((input, textures))
  }
}