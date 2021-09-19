use std::fmt::Display;

pub mod idx;
pub mod btp;
pub mod gfxm;
pub mod gt;
mod bitmap;


#[derive(Debug, PartialEq)]
pub enum Format {
    Btp,
    Bitmap,
    Gfxm,
    GT20,
    Unknown
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      let format_string: &str;
      match self {
          Format::Btp => format_string = "btp",
          Format::Bitmap => format_string = "bmp",
          Format::Gfxm => format_string = "gfxm",
          Format::GT20 => format_string = "gt20",
          Format::Unknown => format_string = "",
      }
      write!(f, "{}", format_string)?;
      Ok(())
    }
}

pub fn identify_format(input: &[u8]) -> Format {
  if btp::parse_magic(input).is_ok() { return Format::Btp };
  if bitmap::parse_magic(input).is_ok() { return Format::Bitmap};
  if gfxm::parse_magic(input).is_ok() { return Format::Gfxm}
  if gt::parse_magic(input).is_ok() { return Format::GT20}
  
  Format::Unknown
}