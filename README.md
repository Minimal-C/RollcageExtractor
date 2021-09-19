# RollcageExtractor
Extracts game files from the video game [Rollcage](https://en.wikipedia.org/wiki/Rollcage_(video_game)).

# Usage

Extracts the contents of Rollcage's IDXData folder. Contents mainly include game textures, models and tracks.

USAGE:

    rollcage-extractor.exe [OPTIONS] <idxFile> [imgFile]

FLAGS:

    -h, --help       Prints help information
    
    -V, --version    Prints version information

OPTIONS:

    -o, --output <path>    Set the output directory of the extracted files

ARGS:

    <idxFile>    The idx file to use.
    
    <imgFile>    The img file to use. Default assumes img file is located in the directory as the idx file.

# Notes
      
Tested extracting Rollcage.img from Rollcage Redux on Windows and Linux.

Extractor automatically generates png images from Rollcage's btp images, they share the same output number.

Track and model (GFXM file) conversion to modern formats currently unsupported.
      
# References
1. http://wiki.xentax.com/index.php/Rollcage_2_IMG
