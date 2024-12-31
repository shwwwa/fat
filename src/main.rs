extern crate bytesize;

use fltk::app::quit;
use fltk::{app, button::Button, frame::Frame, prelude::*, window::Window};
use zip::{CompressionMethod, DateTime};
use unrar::{ListSplit, VolumeInfo};
use clap::{Arg, arg, ArgAction, Command};
use strum_macros::{EnumString, IntoStaticStr};
use serde::{Deserializer, Deserialize};
use serde_derive::Deserialize;
use bytesize::ByteSize;
use time::OffsetDateTime;
use std::{env, ffi::OsStr, fs, fs::File, time::SystemTime, path::PathBuf};
use std::io::{BufReader, Error};
use std::str::FromStr;

/// The difference with file-format lib is that we need as much accurate representation of types as possible,
/// whereas in file_format categories used for quick choice of formats needed for application (why do you need other file's backups for regular app?).
#[derive(Debug, PartialEq, EnumString, IntoStaticStr)]
#[strum(ascii_case_insensitive)]
enum Category {
    /// Files and directories stored in a single, possibly compressed, archive .
    Archive,
    /// Music, sounds, recordings, identifiers of music, music trackers, ringtones, sound card related formats, speech synthesis.
    Audio,
    /// Backup files of applications
    Backup,
    /// Calendar type of files
    Calendar,
    /// Compressed single files or streams.
    Compressed,
    /// Configuration files.
    Config,
    /// Address books and contacts.
    Contacts,
    /// Electronic currencies e.g. bitcoin, gas.
    Currency,
    /// Organized collections of data.
    Database,
    /// Visual information using graphics and spatial relationships.
    Diagram,
    /// Floppy disk images, optical disc images and virtual machine disks.
    Disk,
    /// Word processing and desktop publishing documents.
    Document,
    /// Electronic books.
    Ebook,
    /// Machine-executable code, virtual machine code and shared libraries.
    Executable,
    /// Typefaces used for displaying text on screen or in print.
    Font,
    /// Mathematical formulas.
    Formula,
    /// Game data files (not configs, but saves fe.)
    Gamedata,
    /// Collections of geospatial features, GPS tracks and other location-related files.
    Geospatial,
    /// Haptic effect files
    Haptics,
    /// Help files, man pages, etc..
    Help,
    /// Animations, animated images, raster/vector graphics, icons, cursors.
    Image,
    /// Installer files
    Installer,
    /// Data that provide information about another data.
    Metadata,
    /// 3D images, CAD/CAM drawings, other type of files used for creating and displaying 3D images.
    Model,
    /// Agriculture, etc.. other types of files
    Other,
    /// Collections of files bundled together for easier distribution.
    Package,
    /// Lists of audio or video files, organized in a specific order for sequential playback.
    Playlist,
    /// Slideshows.
    Presentation,
    /// Copies of a read-only memory chip of computers, cartridges, or other electronic devices.
    Rom,
    /// Temporary application files
    Temporary,
    /// Data in tabular form.
    Spreadsheet,
    /// Annotation formats, subtitles and captions.
    Subtitle,
    /// Video stream/container formats, application formats, television broadcast formats.
    Video,
}

impl<'de> Deserialize<'de> for Category {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let category = String::deserialize(de)?;
        Ok(Category::from_str(&category).unwrap_or(Category::Other))
    }
}

#[derive(Deserialize, Debug)]
struct Extension {
    #[allow(dead_code)]
    id: String,
    extension: String,
    name: String,
    category: Category,
    description: String,
    further_reading: String,
    preferred_mime: String,
    mime: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct ExtensionVec {
    extensions: Vec<Extension>,
}

pub struct Arguments {
    file_path: PathBuf,
    extensions_path: PathBuf,
    is_debug: bool,
    is_human: bool,
    only_general: bool,
    ignore_general: bool,
    extension_info: bool
}

/// Is zip file is just a wrapper for other file format.
/// If true, returns id of extension. If false, returns "zip" id.
/// Does not return error, but can return in future.
fn get_complex_zip_id(buf_reader: BufReader<File>) -> Result<String, Error> {
    let mut archive = zip::ZipArchive::new(buf_reader).unwrap();

    // Jar and ear have both MANIFEST.mf file, if it has also application.xml then it's ear, if it does not it's jar
    let mut jar_ear_situation : bool = false;
    for i in 0..archive.len() {
        let file = match archive.by_index(i) {
            Ok(file) => file,
            Err(e) => {
                // We can continue because it still can have some useful files.
                println!("Error when scanning zip - {}", e);
                continue;
            }
        };
        match file.name() {
            "AndroidManifest.xml" => return Ok("apk".to_string()),
            "AppManifest.xaml" => return Ok("xap".to_string()),
            "AppxManifest.xml" => return Ok("appx".to_string()),
            "AppxMetadata/AppxBundleManifest.xml" => return Ok("appxbundle".to_string()),
            "BundleConfig.pb" => return Ok("aab".to_string()),
            "DOMDocument.xml" => return Ok("fla".to_string()),
            "META-INF/AIR/application.xml" => return Ok("air".to_string()),
            "META-INF/application.xml" => return Ok("ear".to_string()),
            "META-INF/MANIFEST.MF" => { jar_ear_situation = true; },
            "META-INF/mozilla.rsa" => return Ok("xpi".to_string()),
            "WEB-INF/web.xml" => return Ok("war".to_string()),
            "doc.kml" => return Ok("kmz".to_string()),
            "document.json" => return Ok("sketch43".to_string()),
            "extension.vsixmanifest" => return Ok("vsix".to_string()),
            _ => 
            {
                if file.name().starts_with("Fusion[Active]/") {
                    return Ok("autodesk123d".to_string())
                } else if file.name().starts_with("circuitdiagram/") {
                    return Ok("cddx".to_string())
                } else if file.name().starts_with("dwf/") {
                    return Ok("dwfx".to_string())
                } else if file.name().ends_with(".fb2") && !file.name().contains('/') {
                    return Ok("fbz".to_string())
                } else if file.name().starts_with("FusionAssetName[Active]/") {
                    return Ok("fusion360".to_string())
                } else if file.name().starts_with("Payload/") && file.name().contains(".app/") {
                    return Ok("ipa".to_string())
                } else if file.name().starts_with("word/") {
                    return Ok("ooxmldocument".to_string())
                } else if file.name().starts_with("visio/") {
                    return Ok("ooxmldrawing".to_string())
                } else if file.name().starts_with("ppt/") {
                    return Ok("ooxmlpresentation".to_string())
                } else if file.name().starts_with("xl/") {
                    return Ok("ooxmlspreadsheet".to_string())
                } else if file.name().starts_with("Documents/") && file.name().ends_with(".fpage") {
                    return Ok("xps".to_string())
                } else if file.name().starts_with("SpaceClaim/") {
                    return Ok("scdoc".to_string())
                } else if file.name().starts_with("3D/") && file.name().ends_with(".model") {
                    return Ok("3mf".to_string())
                } else if (file.name().ends_with(".usd")
                    || file.name().ends_with(".usda")
                    || file.name().ends_with(".usdc"))
                    && !file.name().contains('/') {
                    return Ok("usdz".to_string())
                }
            }
        };
    }
    if jar_ear_situation { return Ok("jar".to_string());}
    // Nothing was found, return regular zip file
    Ok("zip".to_string())
}

/// Is zip file is just a wrapper for other file format.
/// If true, returns extension string. If false, returns "zip" extension.
/// If scanning gives error, returns error.
pub fn get_complex_zip_extension(args: &Arguments, buf_reader: BufReader<File>) -> Result<String, Error> {
    match get_complex_zip_id(buf_reader) {
        Ok(id) => get_extension_from_id(args, id),
        Err(e) => Err(e)
    }
}

/// Gets generic file info like time properties.
fn get_general_info(args: &Arguments){
    println!("## General information:");
    println!("# Name: {}",args.file_path.file_name().unwrap().to_string_lossy());
    
    let metadata = fs::metadata(args.file_path.clone()).unwrap();

    if !args.is_human {println!("# Size: {:?}", metadata.len())}
    else { println!("# Size: {}",ByteSize(metadata.len()).to_string_as(true));}
    // TODO: proper handling of inaccessible time
    let created_time : OffsetDateTime = metadata.created().unwrap_or(SystemTime::now()).into();
    let modified_time : OffsetDateTime = metadata.modified().unwrap_or(SystemTime::now()).into();
    let accessed_time : OffsetDateTime = metadata.accessed().unwrap_or(SystemTime::now()).into();
    
    println!("# Created: {:0>4}-{:0>2}-{:0>2} {:0>2}:{:0>2}:{:0>2}", created_time.year(), created_time.month() as u8, created_time.day(), created_time.hour(), created_time.minute(), created_time.second());
    println!("# Last modified: {:0>4}-{:0>2}-{:0>2} {:0>2}:{:0>2}:{:0>2}", modified_time.year(), modified_time.month() as u8, modified_time.day(), modified_time.hour(), modified_time.minute(), modified_time.second());
    println!("# Last accessed: {:0>4}-{:0>2}-{:0>2} {:0>2}:{:0>2}:{:0>2}", accessed_time.year(), accessed_time.month() as u8, accessed_time.day(), accessed_time.hour(), accessed_time.minute(), accessed_time.second());
    
    if metadata.permissions().readonly() { println!("Readonly");} 
    else { println!("# Readable and writable");}
}

/// Gets `ExtensionVec` by reading Extensions.toml.
fn get_extension_vec(args: &Arguments) -> ExtensionVec{
    let extensions_str = match fs::read_to_string(args.extensions_path.clone()) {
        Ok(c) => c,
        Err(_) => {
            println!("Could not read extensions file: {}", args.extensions_path.to_string_lossy());
            quit();
            unreachable!();
        }
    };

    let extension_vec: ExtensionVec = toml::from_str(&extensions_str).unwrap();
    extension_vec
}

/// Gets extension from it's id from Extensions.toml. Errors if not found.
fn get_extension_from_id(args: &Arguments, id: String) -> Result<String, Error> {
    let mut extension_vec = get_extension_vec(args);
    for extension_data in extension_vec.extensions.iter_mut() {
        if extension_data.id != id {continue};
        return Ok(extension_data.extension.clone());
    }
    Err(Error::new(std::io::ErrorKind::NotFound, "extension was not found by looking through extensions file!"))
}

/// Contain extension data as global to minimize calls --- TODO URGENT
fn get_extension_name(args: &Arguments, extension: &OsStr) -> String {
    let mut extension_vec = get_extension_vec(args);
    for extension_data in extension_vec.extensions.iter_mut() {
        if extension_data.extension != extension.to_str().unwrap() {continue};
        return extension_data.name.clone();
    }
    "unknown type".to_string()
}

/// Gets extension info from Extensions.toml from file.
fn get_extension_info(args: & Arguments, extension: String) {
    println!("## Extension: {}", extension);
    let extensions_str = match fs::read_to_string(args.extensions_path.clone()) {
        Ok(c) => c,
        Err(_) => {
            println!("Could not read extensions file: {}", args.extensions_path.to_string_lossy());
            return;
        }
    };

    let mut extension_vec: ExtensionVec = toml::from_str(&extensions_str).unwrap();
    for extension_data in extension_vec.extensions.iter_mut() {
        if extension_data.extension.ne(&extension) {continue};
        let category : &str = (&extension_data.category).into();
        println!("# Category: {}", category);
        println!("# Name: {}", extension_data.name);
        println!("# Media type (mime): {}", extension_data.preferred_mime);
        
        // Maybe print ids???
        if args.extension_info {
            if extension_data.mime.len() > 1{
                print!("# Other possible media types (mimes): ");
                for mime in extension_data.mime.iter_mut() {
                    if mime == &extension_data.preferred_mime { continue;}
                    print!("{}; ", mime);
                }
                println!();
            }
            println!("# Description: {}", extension_data.description);
            println!("# Further reading: {}", extension_data.further_reading)
        }
    }
}

/// Gets specified rar info about file.
fn get_rar_info(args: &Arguments) {
    println!("## RAR information");
    let mut option = None;
    match unrar::Archive::new(&args.file_path).break_open::<ListSplit>(Some(
        &mut option
    )) {
        // Looks like I need to write my own implementations of rar lib
        Ok(archive) => {
            if archive.has_comment() { println!("# Comment: currently not supported", )}
            if archive.volume_info() != VolumeInfo::None {
                println!("# This is multi-part archive, it is not supported for now.");
                return;
            }
            if let Some(error) = option {
                // If the error's data field holds an OpenArchive, an error occurred while opening,
                // the archive is partly broken (e.g. broken header), but is still readable from.
                // So we continue reading
                println!("Error: {}, continuing.", error);
            }
            for entry in archive {
                match entry {
                    Ok(e) => println!("{}", e),
                    Err(err) => println!("Error: {}", err),
                }
            }    
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

/// Gets specified zip info about file.
fn get_zip_info(args: &Arguments, buf_reader: BufReader<File>) {
    println!("## ZIP information");
    let mut archive = zip::ZipArchive::new(buf_reader).unwrap();
    if !archive.comment().is_empty() {
        println!("# Comment: {:?}", std::str::from_utf8(archive.comment()).unwrap());
    }

    print!("# Compressed size: ");
    let size : u64 = fs::metadata(args.file_path.clone()).unwrap().len();
    let decompressed_size : u64 = archive.decompressed_size().unwrap_or(0).try_into().unwrap();
    let mut percent = (size as f32 / decompressed_size as f32)*100.;
    if percent > 100. { percent = 100.}
    if args.is_human { println!("{}/{} ({:.2}%)", 
        ByteSize(size).to_string_as(true),
        ByteSize(decompressed_size).to_string_as(true),
        percent
    )}
    else { println!("{}/{} ({:.2}%)",size, decompressed_size, percent) ; }
    
    // While we gather zip file information, gather also used compression methods
    let mut compression_methods : Vec<CompressionMethod> = Vec::new();
    println!("# Zip file contains:");
    for i in 0..archive.len() {
        let file = match archive.by_index(i) {
            Ok(file) => file,
            Err(e) => {
                println!("Error (most likely encrypted file): {}", e);
                continue;
            }
        };
        
        if !compression_methods.contains(&file.compression()) {
            compression_methods.push(file.compression());
        }

        let outpath = match file.enclosed_name() {
            Some(path) => path,
            None => {
                println!("File {} has a suspicious path", file.name());
                continue;
            }
        };
        
        // Comment scope
        {
            let comment = file.comment();
            let name = file.name();
            if !comment.is_empty() {
                println!("File {name} has comment: {comment}");
            }
        }

        if file.is_dir() {
            println!(
                "\"{}\"",
                outpath.display()
            );
        } else {
            let last_modified : DateTime = file.last_modified().unwrap_or_default();
            let percent = format!("{:.prec$}", ((file.compressed_size() as f32 / file.size() as f32) * 100.).min(100.), prec = 2) .to_string();
            let file_size : String = if args.is_human {
                ByteSize(file.compressed_size()).to_string_as(true) +
                "/" +
                &ByteSize(file.size()).to_string_as(true) +
                ") (" +
                &percent +
                "%"
            }
            else {
                file.compressed_size().to_string() +
                "/" +
                &file.size().to_string() +
                ") (" +
                &percent +
                "%"
            };
            print!(
                "\"{}\" ({}) ({}) (last modified: {}) ({})",
                outpath.display(),
                file_size,
                get_extension_name(args, file.mangled_name().extension().unwrap_or(OsStr::new(""))),
                last_modified,
                file.crc32()
            );
            
            // Unreachable for now
            if file.encrypted() {
                print!(" (encrypted)");
            }
            println!();
        }
    }
    print!("# Compression methods used: ");
    for method in compression_methods.iter_mut() {
        print!("{} ", method);
    }
    println!()
}

/// Gets info about file.
fn get_info(args: &Arguments) {
    if !args.file_path.exists() { 
        println!("Path to file does not exist.");
        return;
    }
    if !args.file_path.is_file() { 
        println!("Path to file leads to directory, not file.");
        return;
    }

    let file_extension: &std::ffi::OsStr = args.file_path.extension().unwrap_or(OsStr::new(""));
    let buf_reader : BufReader<fs::File> = BufReader::new(fs::File::open(&args.file_path).unwrap());
    
    if !args.ignore_general {get_general_info(args)};
    let mut extension = file_extension.to_str().unwrap_or("").to_string();
    
    // Now we scan zip data to find some complex types.
    if extension.eq("zip") {
        // If it's a zip, we might need to check for more complex zip types
        extension = match get_complex_zip_extension(args, buf_reader) {
            Ok(extension) => extension.to_string(),
            Err(e) => {
                println!("## Unreadable zip file: {}", e);
                "zip".to_string()
            }
        };
    }

    let buf_reader : BufReader<fs::File> = BufReader::new(fs::File::open(&args.file_path).unwrap());
    get_extension_info(args, extension);
    // Specific use-cases (even works for specific files like .apk for listing files)
    if !args.only_general { 
        if file_extension.eq("zip") { get_zip_info(args, buf_reader) }
        else if file_extension.eq("rar") { get_rar_info(args) };
    }
}

/// Boot function.
fn main() {
    // Console arguments
    let m = Command::new("fat")
        .author("caffidev, caffidev@gmail.com")
        .version("0.1.1")
        .about("fat - File Analysis Tool, analyzes metadata and tries to guess its extension.")
        .disable_help_subcommand(true)
        .disable_help_flag(true)
        .arg(arg!(<FILE> ... "File to analyze").value_parser(clap::value_parser!(PathBuf)))
        .arg(
            Arg::new("help")
                .short('?')
                .long("help")
                .action(ArgAction::Help)
                .help("Prints help (this message).")
        )
        .arg(
            Arg::new("extension-info")
            .action(ArgAction::SetTrue)
                .short('e')
                .long("extension-info")
                .help("Provides more info about extension: MIME type, where to read about it etc..")
        )
        .arg(
            Arg::new("debug")
                .action(ArgAction::SetTrue)
                .short('d')
                .long("debug")
                .help("Turns on debugging mode.")
        )
        .arg(
            Arg::new("human")
            .action(ArgAction::SetTrue)
                .short('h')
                .long("human")
                .help("Prints numbers in human-readable way (124 kiB, 76 miB)")
        )
        .arg(
            Arg::new("ignore-general")
            .action(ArgAction::SetTrue)
                .long("ignore-general")
                .short('i')
                .help("Provides only general info e.g name, size, when accessed...")
        )
        .arg(
            Arg::new("only-general")
            .action(ArgAction::SetTrue)
                .long("only-general")
                .short('g')
                .help("Provide only special info e.g basic extension info, special metadata of file... (when with ignore-general provides only info of extension)")
        )
        .after_help("This app was written to analyze files, and give as much info about it as possible")
        .get_matches();
    
    let file_path : PathBuf = m.get_one::<PathBuf>("FILE").unwrap().clone();
    // Getting path to extensions.toml (forced to use env::current_dir())
    let mut extensions_path = env::current_dir().unwrap().clone();
    extensions_path.push("Extensions.toml");

    let args = Arguments
    { 
        file_path,
        extensions_path,
        is_debug : m.get_flag("debug"), 
        is_human : m.get_flag("human"), 
        only_general : m.get_flag("only-general"), 
        ignore_general: m.get_flag("ignore-general"), 
        extension_info: m.get_flag("extension-info") 
    };
    if args.is_debug { println!("Path to file: {:?}",&args.file_path); }

    // GUI interface (for now)
    let app = app::App::default();
    let mut wind = Window::new(100, 100, 400, 300, "FAT-RS v0.1.1");
    Frame::new(0, 0, 400, 200, "Program to analyze files");
    let mut but = Button::new(160, 210, 80, 40, "Load");
    wind.end();
    wind.show();


    // On pressing button we get info about file (selected from above)
    but.set_callback(move |_| get_info(&args));
    
    app.run().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[fixture]
    #[once]
    fn once_fixture() -> Arguments {
        let file_path = PathBuf::from_str(r"samples\recognition\zip\").unwrap();
        // Getting path to extensions.toml (forced to use env::current_dir())
        let mut extensions_path = env::current_dir().unwrap().clone();
        extensions_path.push("Extensions.toml");
        Arguments {
            file_path,
            extensions_path,
            is_debug : true,
            is_human : false, 
            only_general : false,
            ignore_general: false,
            extension_info: false
        }
    }

    #[rstest]
    #[case::threemf("3mf")]
    #[case::one23dx("123dx")]
    #[case::aab("aab")]
    #[case::air("air")]
    #[case::apk("apk")]
    #[case::appx("appx")]
    #[case::appxbundle("appxbundle")]
    #[case::cddx("cddx")]
    #[case::docx("docx")]
    #[case::dwfx("dwfx")]
    #[case::ear("ear")]
    #[case::f3d("f3d")]
    #[case::fbx("fbz")]
    #[case::fla("fla")]
    #[case::ipa("ipa")]
    #[case::jar("jar")]
    #[case::kmz("kmz")]
    #[case::pptx("pptx")]
    #[case::scdoc("scdoc")]
    #[case::sketch("sketch")]
    #[case::usdz("usdz")]
    #[case::vsdx("vsdx")]
    #[case::vsix("vsix")]
    #[case::war("war")]
    #[case::xap("xap")]
    #[case::xlsx("xlsx")]
    #[case::xpi("xpi")]
    #[case::xps("xps")]
    fn recognition_tests(once_fixture: &Arguments, #[case] extension: String){
        let mut file_path = once_fixture.file_path.clone();
        file_path.push(format!("{}.zip", extension));

        let buf_reader : BufReader<fs::File> = BufReader::new(fs::File::open(file_path).unwrap());

        assert_eq!(get_complex_zip_extension(&once_fixture, buf_reader).unwrap(), extension);
    }
}