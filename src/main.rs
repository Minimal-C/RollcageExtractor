use std::path::Path;

use rollcage_file_parser::file_formats::{
    self,
    btp::{parse_btp_header, parse_textures},
    gt::decompress,
    identify_format,
    idx::{parse_records, IdxRecord},
};

extern crate clap;
use clap::{crate_version, App, Arg};

#[derive(Debug)]
struct ImgEntry<'a> {
    record_id: usize,
    record: IdxRecord,
    data: &'a [u8],
    format: file_formats::Format,
}

impl ImgEntry<'_> {
    pub fn save_to_file(&self, output_path: &Path) -> Result<(), String> {
        let data = match self.format {
            file_formats::Format::GT20 => {
                match decompress(self.data, self.record.decompressed_file_length) {
                    Ok(uncompressed_data) => uncompressed_data,
                    Err(e) => {
                        let err_msg = format!(
                            "Could not uncompress image_file for record {}: {:?}",
                            self.record_id, e
                        );
                        return Err(err_msg);
                    }
                }
            }
            _ => {
                // assumed no compression
                self.data.to_vec()
            }
        };

        let data_format = identify_format(&data);
        let filename_stem = output_path.join(format!("output{}", self.record_id));

        if data_format == file_formats::Format::Btp {
            self.save_png_from_btp(&data, &filename_stem);
        }

        let filename = filename_stem.with_extension(data_format.to_string());
        if std::fs::write(filename, data).is_err() {
            println!("Couldn't save the file")
        };

        Ok(())
    }

    fn save_png_from_btp(&self, data: &[u8], filename_stem: &Path) {
        let btp_header = parse_btp_header(data).unwrap().1;
        let textures = parse_textures(data, &btp_header).unwrap().1;
        if !textures.is_empty() {
            match std::fs::create_dir(filename_stem) {
                Ok(_) => (),
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::AlreadyExists {
                        panic!(
                            "Could not create output folder during btp conversion: {}",
                            e
                        );
                    }
                }
            };
        }
        for (i, texture) in textures.iter().enumerate() {
            let image = texture.to_rgba_image();
            let filename_bitmap = filename_stem
                .join(format!("image_{}", i))
                .with_extension("bmp");
            match image.save(&filename_bitmap) {
                Ok(_) => (),
                Err(e) => {
                    println!("Could not save {:?}:{}", filename_bitmap, e);
                }
            }
        }
    }
}

// Check if the specified file exists.
fn is_file(val: String) -> Result<(), String> {
    if std::path::Path::is_file(std::path::Path::new(&val)) {
        Ok(())
    } else {
        Err(format!("Specified path is not a file. {}", val))
    }
}

// Check if the specified directory exists, if not attempt to create directory.
fn validate_dir(val: String) -> Result<(), String> {
    let val = std::path::Path::new(&val);
    if std::path::Path::is_dir(val) {
        Ok(())
    } else {
        match std::fs::create_dir(&val) {
            Ok(_) => Ok(()),
            Err(e) => {
                if e.kind() != std::io::ErrorKind::AlreadyExists {
                    Err(format!("Could not create output folder: {}", e))
                } else {
                    Ok(())
                }
            }
        }
    }
}

fn main() {
    let matches = App::new("RollCage Extractor")
  .version(crate_version!())
  .about("Extracts the contents of Rollcage's IDXData folder. Contents mainly include game textures, models and tracks.")
  .arg(Arg::with_name("idxFile")
    .help("The idx file to use.")
    .required(true)
    .index(1)
    .validator(is_file))
  .arg(Arg::with_name("imgFile")
    .help("The img file to use. Default assumes img file is located in the directory as the idx file.")
    .required(false)
    .validator(is_file))
  .arg(Arg::with_name("output")
    .help("Set the output directory of the extracted files")
    .short("o")
    .long("output")
    .value_name("path")
    .validator(validate_dir))
  .get_matches();

    let idx_path = std::path::Path::new(matches.value_of("idxFile").unwrap());

    let img_pathbuf = match matches.value_of("imgFile") {
        Some(v) => std::path::PathBuf::from(v),
        None => {
            let out = idx_path.with_extension("img");
            if out.is_file() {
                out
            } else {
                let panic_message = format!(
                    "Could not find a file with the path: {}",
                    out.to_str().unwrap()
                );
                panic!("{}", panic_message);
            }
        }
    };

    let img_path = img_pathbuf.as_path();
    let output = matches.value_of("output").unwrap_or(".");
    let output_path = std::path::Path::new(output);

    let img = match std::fs::read(img_path) {
        Ok(data) => data,
        Err(e) => {
            panic!("Failed to read {:?}: {}. Cannot continue.", img_path, e);
        }
    };
    let idx = match std::fs::read(idx_path) {
        Ok(data) => data,
        Err(e) => {
            panic!("Failed to read {:?}: {}. Cannot continue.", idx_path, e);
        }
    };

    let records = parse_records(&idx).unwrap().1;

    let mut img_entries: Vec<ImgEntry> = Vec::new();
    for (record_id, record) in records.iter().enumerate() {
        let record_data = img
            .get(
                record.file_offset as usize
                    ..(record.file_offset + record.compressed_file_length) as usize,
            )
            .unwrap();
        let format = identify_format(record_data);
        let img_entry = ImgEntry {
            record_id,
            record: *record,
            data: record_data,
            format,
        };
        img_entries.push(img_entry);
    }

    for (i, img_entry) in img_entries.iter().enumerate() {
        match img_entry.save_to_file(output_path) {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e)
            }
        }
        print!("\rExtracting: [{}/{}] ", i + 1, img_entries.len());
    }
    println!("\nDone!");
}