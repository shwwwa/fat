extern crate bytesize;
use std::fs::File;
use std::io::BufReader;
use std::{env, ffi::OsStr, fs, time::SystemTime, path::PathBuf};
use clap::{Arg, ArgAction, Command};
use fltk::{app, button::Button, frame::Frame, prelude::*, window::Window};
use bytesize::ByteSize;
use time::OffsetDateTime;

struct Arguments {
    is_debug: bool,
    is_human: bool
}

/// Currently returns not an extension, but if it is a zip file.
fn get_general_info(args: &Arguments, path : &PathBuf) -> bool{
    println!("## General information:");
    println!("# Name: {}",path.file_name().unwrap().to_string_lossy());
    // If file is zip
    let is_zip_file: bool = path.extension().unwrap_or(OsStr::new("")).eq("zip");
    let metadata = fs::metadata(path).unwrap();

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

    is_zip_file
}

fn get_zip_info(args: &Arguments, buf_reader: BufReader<File>) {
    let mut archive = zip::ZipArchive::new(buf_reader).unwrap();
    // TODO: test it properly 
    if !archive.comment().is_empty() {
        print!("# Comment: {:?}", archive.comment());
    }

    print!("# Decompressed size: ");
    let decompressed_size : u64 = archive.decompressed_size().unwrap_or(0).try_into().unwrap();
    if args.is_human { println!("{}", ByteSize(decompressed_size).to_string_as(true))}
    else { println!("{}", decompressed_size); }
    
    println!("# Zip file contains:");
    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap();
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
            let file_size : String;
            if args.is_human {
                file_size = ByteSize(file.size()).to_string_as(true);
            }
            else {
                file_size = file.size().to_string();
            }
            println!(
                "\"{}\" ({})",
                outpath.display(),
                file_size
            );
        }
    }
}

fn get_info(args: &Arguments, path : &PathBuf) {
    if !path.exists() { println!("Path to file does not exist.");}
    if !path.is_file() { println!("Path to file leads to directory, not file");}

    let buf_reader : BufReader<fs::File>;
    buf_reader = BufReader::new(fs::File::open(&path).unwrap());
    let is_zip_file = get_general_info(&args, path);

    if is_zip_file {
        println!("# Format - zip\n## Zip information");
        get_zip_info(&args, buf_reader);
    }

}

fn main() {
    // Console arguments
    let m = Command::new("File Analysis Tool")
        .author("caffidev, caffidev@gmail.com")
        .version("0.1.1")
        .about("FAT - File Analysis Tool, analyzes file, tries to guess its extension etc..")
        .disable_help_subcommand(true)
        .disable_help_flag(true)
        .arg(
            Arg::new("debug")
                .action(ArgAction::SetTrue)
                .short('d')
                .long("debug")
                .help("Turns on debugging mode")
        )
        // Change help short flag to `?`
        .arg(
            Arg::new("help")
                .short('?')
                .long("help")
                .action(ArgAction::Help)
                .help("Print help")
        )
        .arg(
            Arg::new("human")
            .action(ArgAction::SetTrue)
                .short('h')
                .long("human")
                .help("Prints numbers in human-readable way (124K, 76M)")
        )
        .after_help("This app was written to analyze files, and give as much info about it as possible")
        .get_matches();
    
    // TODO: proper config system
    let args = Arguments{ is_debug : m.get_flag("debug"), is_human : m.get_flag("human") };
    let app = app::App::default();
    let mut wind = Window::new(100, 100, 400, 300, "FAT-RS v0.1.1");
    Frame::new(0, 0, 400, 200, "Program to analyze files");
    let mut but = Button::new(160, 210, 80, 40, "Load");
    wind.end();
    wind.show();

    // Getting info
    let mut zip_path = env::current_dir().unwrap();
    zip_path.push(r"test_files\test.zip");
    if args.is_debug { println!("Path to file: {:?}",zip_path); }

    // On pressing button we get info about file (selected from above)
    but.set_callback(move |_| get_info(&args, &zip_path));
    
    app.run().unwrap();
}