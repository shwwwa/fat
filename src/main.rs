extern crate bytesize;
use std::io::{prelude::*, BufReader};
use std::{env, ffi::OsStr, fs, time::SystemTime};
use clap::{Arg, ArgAction, Command};
use fltk::{app, button::Button, frame::Frame, prelude::*, window::Window};
use bytesize::ByteSize;
use time::OffsetDateTime;

fn main() {
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
    
    let is_debug : bool = m.get_flag("debug");
    let is_human : bool = m.get_flag("human");

    let app = app::App::default();
    let mut wind = Window::new(100, 100, 400, 300, "ETS2 save editor v0.1.1");
    let mut frame = Frame::new(0, 0, 400, 200, "");
    let mut but = Button::new(160, 210, 80, 40, "Click me!");
    wind.end();
    wind.show();
    but.set_callback(move |_| frame.set_label("Hello World!")); // the closure capture is mutable borrow to our button
    let mut zip_path = env::current_dir().unwrap();
    zip_path.push(r"test_files\test.zip");
    println!("{:?}",zip_path);
    if zip_path.exists() && zip_path.is_file() {
        print!("{} - ",zip_path.file_name().unwrap().to_string_lossy());
        // If file is zip (currently guaranteed)
        let is_zip_file: bool = zip_path.extension().unwrap_or(OsStr::new("")).eq("zip");
        let buf_reader : BufReader<fs::File>;
        buf_reader = BufReader::new(fs::File::open(&zip_path).unwrap());
        let metadata = fs::metadata(zip_path).unwrap();

        if !is_human {println!("{:?}", metadata.len())}
        else { print!("{} - ",ByteSize(metadata.len()).to_string_as(true));}
        // TODO: proper handling of inaccessible time
        let created_time : OffsetDateTime = metadata.created().unwrap_or(SystemTime::now()).into();
        let modified_time : OffsetDateTime = metadata.modified().unwrap_or(SystemTime::now()).into();
        let accessed_time : OffsetDateTime = metadata.accessed().unwrap_or(SystemTime::now()).into();
        
        print!("created: {:0>4}-{:0>2}-{:0>2} {:0>2}:{:0>2}:{:0>2} - ", created_time.year(), created_time.month() as u8, created_time.day(), created_time.hour(), created_time.minute(), created_time.second());
        print!("last modified: {:0>4}-{:0>2}-{:0>2} {:0>2}:{:0>2}:{:0>2} - ", modified_time.year(), modified_time.month() as u8, modified_time.day(), modified_time.hour(), modified_time.minute(), modified_time.second());
        print!("last accessed: {:0>4}-{:0>2}-{:0>2} {:0>2}:{:0>2}:{:0>2} - ", accessed_time.year(), accessed_time.month() as u8, accessed_time.day(), accessed_time.hour(), accessed_time.minute(), accessed_time.second());
        
        if metadata.permissions().readonly() { println!("readonly");} 
        else { print!("read&writeable - ");}

        if is_zip_file {
            let mut archive = zip::ZipArchive::new(buf_reader).unwrap();
            // TODO: test it properly 
            if !archive.comment().is_empty() {
                print!("comment: {:?} - ", archive.comment());
            }

            println!("Zip file contains:");
            for i in 0..archive.len() {
                let file = archive.by_index(i).unwrap();
                let outpath = match file.enclosed_name() {
                    Some(path) => path,
                    None => {
                        println!("Entry {} has a suspicious path", file.name());
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
                    if is_human {
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
    }

    app.run().unwrap();
}