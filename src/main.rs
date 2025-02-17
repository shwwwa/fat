extern crate bytesize;

mod components;
mod rar;
mod tests;
mod zip;

use crate::components::{Arguments, ExtensionVec};
use bytesize::ByteSize;
use clap::{arg, Arg, ArgAction, ArgMatches, Command};

use std::{
    env,
    ffi::OsStr,
    fs,
    io::{BufReader, Error},
    path::PathBuf,
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
        println!("Readable and writable");
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
            std::process::exit(-1);
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
        if args.more_info {
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

/// Check file's path for availability, returns error if not successful.
fn check_file_path(args: &Arguments) -> Result<(), Error> {
    match args.file_path.try_exists() {
        Ok(_) => {}
        Err(e) => return Err(e),
    }

    if !args.file_path.is_file() {
        return Err(Error::new(
            std::io::ErrorKind::IsADirectory,
            "file is a directory",
        ));
    }

    Ok(())
}

/// CLI builder.
fn cli() -> Command {
    Command::new("fat")
        .about("fat - file analysis tool, analyzes files and provides required info")
        .author("caffidev, caffidev@gmail.com")
        .version("0.5.1")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand_value_name("SUBCOMMAND")
        .subcommand_help_heading("Subcommands")
        .disable_help_subcommand(true)
        // for overriding -h
        .disable_help_flag(true)
        .disable_version_flag(true)
        .arg(
            Arg::new("help")
                .short('?')
                .long("help")
                .action(ArgAction::Help)
                .help("prints help (this message)")
        )
        .arg(
            Arg::new("version")
                .short('V')
                .long("version")
                .action(ArgAction::Version)
                .help("prints version")
        )
        .arg(
            Arg::new("byte")
                .action(ArgAction::SetTrue)
                .short('b')
                .long("byte")
                .help("prints byte-sized values instead of human-readable ones")
        )
        .arg(
            Arg::new("debug")
                .action(ArgAction::SetTrue)
                .short('d')
                .long("debug")
                .help("prints debug information when running commands")
        )
        .subcommand(
            Command::new("recognize")
                .about("provides info about extension, tries to guess extension if not provided")
                .arg(arg!(<FILE> ... "File to recognize").value_parser(clap::value_parser!(PathBuf)))
                .arg_required_else_help(true)
                .arg(
                    Arg::new("analyze")
                        .action(ArgAction::SetTrue)
                        .short('a')
                        .long("analyze")
                        .help("prints more info about extension after recognition")
                )
                ,
        )
        .subcommand(
            Command::new("analyze")
                .about("analyzes files for strange things in it, extracts any data from it that does not belong in it, searches for sfx, encryption")
                .arg(arg!(<FILE> ... "File to analyze").value_parser(clap::value_parser!(PathBuf)))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("test")
                .about("test for file's errors and strange things")
                .arg(arg!(<FILE> ... "File to test").value_parser(clap::value_parser!(PathBuf)))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("general")
                .about("provides general properties contained in file, chagnes it on demand")
                .arg(arg!(<FILE> ... "File to analyze").value_parser(clap::value_parser!(PathBuf)))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("metadata")
                .about("provides metadata contained in file, changes it on demand")
                .arg(arg!(<FILE> ... "File to analyze").value_parser(clap::value_parser!(PathBuf)))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("data")
                .about("provides data contained in file, extracts it on demand")
                .arg(arg!(<FILE> ... "File to analyze").value_parser(clap::value_parser!(PathBuf)))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("check")
                .about("currently does nothing")
        )
        // Helper subcommands
        .subcommand(
            Command::new("help")
                .hide(true)
                .about("provides help message for subcommand (todo)")
        )
        .subcommand(
            Command::new("version")
                .hide(true)
                .about("provides version of modules (todo)")
        )
        .after_help("--list about helper subcommands (todo) \ncheck man pages or <link-to-guthub> for more info")
}

/// Initializes `Arguments` based on subcommand `ArgMatches` and options `ArgMatches`
fn initialize(arg_m: &ArgMatches, sub_m: &ArgMatches) -> Arguments {
    let file_path: PathBuf = sub_m.get_one::<PathBuf>("FILE").unwrap().clone();
    let mut extensions_path = env::current_dir().unwrap().clone();
    extensions_path.push("Extensions.toml");

    let mut more_info: bool = false;
    if sub_m.try_contains_id("analyze").is_ok() {
        more_info = sub_m.get_flag("analyze");
    } else if arg_m.subcommand_name().unwrap() == "analyze" {
        more_info = true
    }

    let args = Arguments {
        file_path: file_path.clone(),
        extensions_path: extensions_path.clone(),
        is_debug: arg_m.get_flag("debug"),
        is_human: !arg_m.get_flag("byte"),
        more_info,
    };

    if args.is_debug {
        println!("File path: {}", file_path.to_string_lossy());
        println!("Extensions path: {}", extensions_path.to_string_lossy());
    }

    args
}

/// Boot function for CLI.
/// Todo: implement proper ExitCode
fn main() -> Result<(), Error> {
    let arg_m = cli().get_matches();

    match arg_m.subcommand() {
        Some(("recognize", sub_m)) => {
            // Basic initialization
            let args = initialize(&arg_m, sub_m);
            match check_file_path(&args) {
                Ok(()) => {}
                Err(e) => {
                    println!("Error happened when executing recognize command: {:#?}", e);
                    return Ok(());
                }
            }

            let file_extension: &std::ffi::OsStr =
                args.file_path.extension().unwrap_or(OsStr::new(""));
            let buf_reader: BufReader<fs::File> =
                BufReader::new(fs::File::open(&args.file_path).unwrap());

            let mut extension = file_extension.to_str().unwrap_or("").to_string();

            // Todo: add more types to scan besides zip one.
            // Now we scan zip data to find some complex types.
            if extension.eq("zip") || extension.is_empty() {
                // If it's a zip, we might need to check for more complex zip types
                extension = match crate::zip::get_complex_zip_extension(&args, buf_reader) {
                    Ok(extension) => extension.to_string(),
                    Err(e) => {
                        println!("Could not recognize file as zip: {}", e);
                        "zip".to_string()
                    }
                };
            }

            get_extension_info(&args, extension);

            Ok(())
        }
        // Currently only thing it does it prints info about extension.
        Some(("analyze", sub_m)) => {
            let args = initialize(&arg_m, sub_m);
            match check_file_path(&args) {
                Ok(()) => {}
                Err(e) => {
                    println!("Error happened when executing analyze command: {:#?}", e);
                    return Ok(());
                }
            }

            let file_extension: &std::ffi::OsStr =
                args.file_path.extension().unwrap_or(OsStr::new(""));

            let extension = file_extension.to_str().unwrap_or("").to_string();

            get_extension_info(&args, extension);
            Ok(())
        }
        Some(("test", _)) => {
            println!("currently not implemented");
            Ok(())
        }
        Some(("general", sub_m)) => {
            let args = initialize(&arg_m, sub_m);
            match check_file_path(&args) {
                Ok(()) => {}
                Err(e) => {
                    println!("Error happened when executing recognize command: {:#?}", e);
                    return Ok(());
                }
            }

            get_general_info(&args);
            Ok(())
        }
        Some(("metadata", sub_m)) => {
            // Currently is the same as data
            let args = initialize(&arg_m, sub_m);
            match check_file_path(&args) {
                Ok(()) => {}
                Err(e) => {
                    println!("Error happened when executing recognize command: {:#?}", e);
                    return Ok(());
                }
            }

            let file_extension: &std::ffi::OsStr =
                args.file_path.extension().unwrap_or(OsStr::new(""));
            let buf_reader: BufReader<fs::File> =
                BufReader::new(fs::File::open(&args.file_path).unwrap());

            // Specific use-cases (even works for specific files like .apk for listing files)
            if file_extension.eq("zip") {
                crate::zip::get_zip_info(&args, buf_reader)
            } else if file_extension.eq("rar") {
                crate::rar::get_rar_info(&args)
            } else {
                println!("We can't still extract data from anything that is zip or rar archive.");
            }

            Ok(())
        }
        Some(("data", sub_m)) => {
            let args = initialize(&arg_m, sub_m);
            match check_file_path(&args) {
                Ok(()) => {}
                Err(e) => {
                    println!("Error happened when executing recognize command: {:#?}", e);
                    return Ok(());
                }
            }

            let file_extension: &std::ffi::OsStr =
                args.file_path.extension().unwrap_or(OsStr::new(""));
            let buf_reader: BufReader<fs::File> =
                BufReader::new(fs::File::open(&args.file_path).unwrap());

            // Specific use-cases (even works for specific files like .apk for listing files)
            if file_extension.eq("zip") {
                crate::zip::get_zip_info(&args, buf_reader)
            } else if file_extension.eq("rar") {
                crate::rar::get_rar_info(&args)
            } else {
                println!("We can't still extract data from anything that is zip or rar archive.");
            }
            Ok(())
        }
        Some(("check", _)) => {
            println!("currently does nothing");
            Ok(())
        }
        _ => {
            println!("help and version are currently unused as of v0.5.1");
            Ok(())
        }
    }
}
