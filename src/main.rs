extern crate bytesize;

mod components;
mod rar;
mod tests;
mod zip;

use crate::components::{Arguments, ExtensionVec};
use bytesize::ByteSize;
use clap::{arg, Arg, ArgAction, Command};
use fltk::app::quit;
use fltk::{app, button::Button, frame::Frame, prelude::*, window::Window};
#[allow(unused_imports)]
use std::{
    env,
    ffi::OsStr,
    fs,
    fs::File,
    io::{BufReader, Error},
    path::PathBuf,
    str::FromStr,
    time::SystemTime,
};
use time::OffsetDateTime;

/// Gets generic file info like time properties.
fn get_general_info(args: &Arguments) {
    println!("## General information:");
    println!(
        "# Name: {}",
        args.file_path.file_name().unwrap().to_string_lossy()
    );

    let metadata = fs::metadata(args.file_path.clone()).unwrap();

    if !args.is_human {
        println!("# Size: {:?}", metadata.len())
    } else {
        println!("# Size: {}", ByteSize(metadata.len()).to_string_as(true));
    }
    // TODO: proper handling of inaccessible time
    let created_time: OffsetDateTime = metadata.created().unwrap_or(SystemTime::now()).into();
    let modified_time: OffsetDateTime = metadata.modified().unwrap_or(SystemTime::now()).into();
    let accessed_time: OffsetDateTime = metadata.accessed().unwrap_or(SystemTime::now()).into();

    println!(
        "# Created: {:0>4}-{:0>2}-{:0>2} {:0>2}:{:0>2}:{:0>2}",
        created_time.year(),
        created_time.month() as u8,
        created_time.day(),
        created_time.hour(),
        created_time.minute(),
        created_time.second()
    );
    println!(
        "# Last modified: {:0>4}-{:0>2}-{:0>2} {:0>2}:{:0>2}:{:0>2}",
        modified_time.year(),
        modified_time.month() as u8,
        modified_time.day(),
        modified_time.hour(),
        modified_time.minute(),
        modified_time.second()
    );
    println!(
        "# Last accessed: {:0>4}-{:0>2}-{:0>2} {:0>2}:{:0>2}:{:0>2}",
        accessed_time.year(),
        accessed_time.month() as u8,
        accessed_time.day(),
        accessed_time.hour(),
        accessed_time.minute(),
        accessed_time.second()
    );

    if metadata.permissions().readonly() {
        println!("Readonly");
    } else {
        println!("# Readable and writable");
    }
}

/// Gets `ExtensionVec` by reading Extensions.toml.
fn get_extension_vec(args: &Arguments) -> ExtensionVec {
    let extensions_str = match fs::read_to_string(args.extensions_path.clone()) {
        Ok(c) => c,
        Err(_) => {
            println!(
                "Could not read extensions file: {}",
                args.extensions_path.to_string_lossy()
            );
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
        if extension_data.id != id {
            continue;
        };
        return Ok(extension_data.extension.clone());
    }
    Err(Error::new(
        std::io::ErrorKind::NotFound,
        "extension was not found by looking through extensions file!",
    ))
}

/// Contain extension data as global to minimize calls --- TODO URGENT
fn get_extension_name(args: &Arguments, extension: &OsStr) -> String {
    let mut extension_vec = get_extension_vec(args);
    for extension_data in extension_vec.extensions.iter_mut() {
        if extension_data.extension != extension.to_str().unwrap() {
            continue;
        };
        return extension_data.name.clone();
    }
    "unknown type".to_string()
}

/// Gets extension info from Extensions.toml from file.
fn get_extension_info(args: &Arguments, extension: String) {
    println!("## Extension: {}", extension);
    let extensions_str = match fs::read_to_string(args.extensions_path.clone()) {
        Ok(c) => c,
        Err(_) => {
            println!(
                "Could not read extensions file: {}",
                args.extensions_path.to_string_lossy()
            );
            return;
        }
    };

    let mut extension_vec: ExtensionVec = toml::from_str(&extensions_str).unwrap();
    for extension_data in extension_vec.extensions.iter_mut() {
        if extension_data.extension.ne(&extension) {
            continue;
        };
        let category: &str = (&extension_data.category).into();
        println!("# Category: {}", category);
        println!("# Name: {}", extension_data.name);
        println!("# Media type (mime): {}", extension_data.preferred_mime);

        // Maybe print ids???
        if args.extension_info {
            if extension_data.mime.len() > 1 {
                print!("# Other possible media types (mimes): ");
                for mime in extension_data.mime.iter_mut() {
                    if mime == &extension_data.preferred_mime {
                        continue;
                    }
                    print!("{}; ", mime);
                }
                println!();
            }
            println!("# Description: {}", extension_data.description);
            println!("# Further reading: {}", extension_data.further_reading)
        }
    }
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
    let buf_reader: BufReader<fs::File> = BufReader::new(fs::File::open(&args.file_path).unwrap());

    if !args.ignore_general {
        get_general_info(args)
    };
    let mut extension = file_extension.to_str().unwrap_or("").to_string();

    // Now we scan zip data to find some complex types.
    if extension.eq("zip") {
        // If it's a zip, we might need to check for more complex zip types
        extension = match crate::zip::get_complex_zip_extension(args, buf_reader) {
            Ok(extension) => extension.to_string(),
            Err(e) => {
                println!("## Unreadable zip file: {}", e);
                "zip".to_string()
            }
        };
    }

    let buf_reader: BufReader<fs::File> = BufReader::new(fs::File::open(&args.file_path).unwrap());
    get_extension_info(args, extension);
    // Specific use-cases (even works for specific files like .apk for listing files)
    if !args.only_general {
        if file_extension.eq("zip") {
            crate::zip::get_zip_info(args, buf_reader)
        } else if file_extension.eq("rar") {
            crate::rar::get_rar_info(args)
        };
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
            Arg::new("gui")
            .action(ArgAction::SetTrue)
                .short('g')
                .long("gui")
                .help("Opens W.I.P GUI mode.")
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
                .short('o')
                .help("Provide only special info e.g basic extension info, special metadata of file... (when with ignore-general provides only info of extension)")
        )
        .after_help("This app was written to analyze files, and give as much info about it as possible")
        .get_matches();

    let file_path: PathBuf = m.get_one::<PathBuf>("FILE").unwrap().clone();
    // Getting path to extensions.toml (forced to use env::current_dir())
    let mut extensions_path = env::current_dir().unwrap().clone();
    extensions_path.push("Extensions.toml");

    let args = Arguments {
        file_path,
        extensions_path,
        gui: m.get_flag("gui"),
        is_debug: m.get_flag("debug"),
        is_human: m.get_flag("human"),
        only_general: m.get_flag("only-general"),
        ignore_general: m.get_flag("ignore-general"),
        extension_info: m.get_flag("extension-info"),
    };
    if args.is_debug {
        println!("Path to file: {:?}", &args.file_path);
    }

    if args.gui {
        let app = app::App::default();
        let mut wind = Window::new(100, 100, 400, 300, "FAT-RS v0.1.1");
        Frame::new(0, 0, 400, 200, "Program to analyze files");
        let mut but = Button::new(160, 210, 80, 40, "Load");
        wind.end();
        wind.show();

        // On pressing button we get info about file (selected from above)
        but.set_callback(move |_| get_info(&args));

        app.run().unwrap();
    } else {
        get_info(&args);
    }
}
