extern crate bytesize;

mod components;
mod rar;
mod tests;
mod zip;

use crate::components::{Arguments, ExtensionVec};
use bytesize::ByteSize;
use clap::{arg, Arg, ArgAction, Command};
use fltk::app::quit;
use fltk::utils::oncelock::Lazy;
use fltk::{enums::*, menu, text, dialog, window};
use fltk::{app, prelude::*, group};

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

/** GUI */

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;
static STATE: Lazy<app::GlobalState<State>> = Lazy::new(app::GlobalState::<State>::get);

pub struct State {
    pub saved: bool,
    pub buffer: text::TextBuffer,
    pub current_file: PathBuf,
}

impl State {
    fn new(buffer: text::TextBuffer) -> Self {
        State {
            saved: true,
            buffer,
            current_file: PathBuf::new(),
        }
    }
}

fn init_menu(m: &mut menu::SysMenuBar) {
    m.add(
        "&File/&New...\t",
        Shortcut::Ctrl | 'n',
        menu::MenuFlag::Normal,
        menu_cb,
    );
    m.add(
        "&File/&Open...\t",
        Shortcut::Ctrl | 'o',
        menu::MenuFlag::Normal,
        menu_cb,
    );
    m.add(
        "&File/&Save\t",
        Shortcut::Ctrl | 's',
        menu::MenuFlag::Normal,
        menu_cb,
    );
    m.add(
        "&File/Save &as...\t",
        Shortcut::Ctrl | 'w',
        menu::MenuFlag::MenuDivider,
        menu_cb,
    );
    let idx = m.add(
        "&File/&Quit\t",
        Shortcut::Ctrl | 'q',
        menu::MenuFlag::Normal,
        menu_cb,
    );
    m.at(idx).unwrap().set_label_color(Color::Red);
    m.add(
        "&Edit/Cu&t\t",
        Shortcut::Ctrl | 'x',
        menu::MenuFlag::Normal,
        menu_cb,
    );
    m.add(
        "&Edit/&Copy\t",
        Shortcut::Ctrl | 'c',
        menu::MenuFlag::Normal,
        menu_cb,
    );
    m.add(
        "&Edit/&Paste\t",
        Shortcut::Ctrl | 'v',
        menu::MenuFlag::Normal,
        menu_cb,
    );
    m.add(
        "&Analyze\t",
        Shortcut::Shift | 'a',
        menu::MenuFlag::Normal,
        menu_cb,
    );
    m.add(
        "&Help/&About\t",
        Shortcut::None,
        menu::MenuFlag::Normal,
        menu_cb,
    );
}

pub fn center() -> (i32, i32) {
    (
        (app::screen_size().0 / 2.0) as i32,
        (app::screen_size().1 / 2.0) as i32, 
    )
}

fn nfc_get_file(mode: dialog::NativeFileChooserType) -> Option<PathBuf> {
    let mut nfc = dialog::NativeFileChooser::new(mode);
    if mode == dialog::NativeFileChooserType::BrowseSaveFile {
        nfc.set_option(dialog::NativeFileChooserOptions::SaveAsConfirm);
    } else if mode == dialog::NativeFileChooserType::BrowseFile {
        nfc.set_option(dialog::NativeFileChooserOptions::NoOptions);
        nfc.set_filter("*.{txt,rs,toml}");
    }
    match nfc.try_show() {
        Err(e) => {
            eprintln!("{}", e);
            None
        }
        Ok(a) => match a {
            dialog::NativeFileChooserAction::Success => {
                let name = nfc.filename();
                if name.as_os_str().is_empty() {
                    dialog::message_title("fat 0.3.0");
                    dialog::alert(center().0 - 200, center().1 - 100, "Specify a file for plain text analyze!");
                    None
                } else {
                    Some(name)
                }
            }
            dialog::NativeFileChooserAction::Cancelled => None,
        },
    }
}

fn quit_cb() {
    STATE.with(|s| {
        if s.saved {
            app::quit();
        } else {
            dialog::message_title("fat 0.3.0");
            let c = dialog::choice2_default(
                "Are you sure you want to exit without saving?",
                "&Yes",
                "&No",
                "",
            );
            if c == Some(0) {
                app::quit();
            }
        }
    });
}

fn win_cb(_w: &mut window::Window) {
    if app::event() == Event::Close {
        quit_cb();
    }
}

fn editor_cb(_e: &mut text::TextEditor) {
    STATE.with(|s| s.saved = false);
}

fn handle_drag_drop(editor: &mut text::TextEditor) {
    editor.handle({
        let mut dnd = false;
        let mut released = false;
        let buf = editor.buffer().unwrap();
        move |_, ev| match ev {
            Event::DndEnter => {
                dnd = true;
                true
            }
            Event::DndDrag => true,
            Event::DndRelease => {
                released = true;
                true
            }
            Event::Paste => {
                if dnd && released {
                    let path = app::event_text();
                    let path = path.trim();
                    let path = path.replace("file://", "");
                    let path = std::path::PathBuf::from(&path);
                    if path.exists() {
                        // we use a timeout to avoid pasting the path into the buffer
                        app::add_timeout3(0.0, {
                            let mut buf = buf.clone();
                            move |_| match buf.load_file(&path) {
                                Ok(_) => (),
                                Err(e) => dialog::alert_default(&format!(
                                    "An issue occured while loading the file: {e}"
                                )),
                            }
                        });
                    }
                    dnd = false;
                    released = false;
                    true
                } else {
                    false
                }
            }
            Event::DndLeave => {
                dnd = false;
                released = false;
                true
            }
            _ => false,
        }
    });
}

fn menu_cb(m: &mut impl MenuExt) {
    if let Ok(mpath) = m.item_pathname(None) {
        let ed: text::TextEditor = app::widget_from_id("ed").unwrap();
        match mpath.as_str() {
            "&File/&New...\t" => {
                STATE.with(|s| {
                    if !s.buffer.text().is_empty() {
                        dialog::message_title("fat 0.3.0");
                        let c = dialog::choice2_default(
                            "Are you sure you want to clear the buffer?",
                            "&Yes",
                            "&No",
                            "",
                        );
                        if c == Some(0) {
                            s.buffer.set_text("");
                            s.saved = false;
                        }
                    }
                });
            }
            "&File/&Open...\t" => {
                if let Some(c) = nfc_get_file(dialog::NativeFileChooserType::BrowseFile) {
                    if let Ok(text) = std::fs::read_to_string(&c) {
                        STATE.with(move |s| {
                            s.buffer.set_text(&text);
                            s.saved = false;
                            s.current_file = c.clone();
                        });
                    }
                }
            }
            "&File/&Save\t" => {
                STATE.with(|s| {
                    if !s.saved && s.current_file.exists() {
                        std::fs::write(&s.current_file, s.buffer.text()).ok();
                    }
                });
            }
            "&File/Save &as...\t" => {
                if let Some(c) = nfc_get_file(dialog::NativeFileChooserType::BrowseSaveFile) {
                    STATE.with(move |s| {
                        std::fs::write(&c, s.buffer.text()).ok();
                        s.saved = true;
                        s.current_file = c.clone();
                    });
                }
            }
            "&File/&Quit\t" => quit_cb(),
            "&Edit/Cu&t\t" => ed.cut(),
            "&Edit/&Copy\t" => ed.copy(),
            "&Edit/&Paste\t" => ed.paste(),
            "&Analyze\t" => { dialog::message_default("Just made for testing purposes.") }
            "&Help/&About\t" => {
                dialog::message_title("fat 0.3.0");
                dialog::message_default("A plain text editor made for rat.")
            }
            _ => unreachable!(),
        }
    }
}

/// Boot function.
fn main() {
    // Console arguments
    let argm = Command::new("fat")
        .author("caffidev, caffidev@gmail.com")
        .version("0.3.1")
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

    let file_path: PathBuf = argm.get_one::<PathBuf>("FILE").unwrap().clone();
    // Getting path to extensions.toml (forced to use env::current_dir())
    let mut extensions_path = env::current_dir().unwrap().clone();
    extensions_path.push("Extensions.toml");

    let args = Arguments {
        file_path,
        extensions_path,
        gui: argm.get_flag("gui"),
        is_debug: argm.get_flag("debug"),
        is_human: argm.get_flag("human"),
        only_general: argm.get_flag("only-general"),
        ignore_general: argm.get_flag("ignore-general"),
        extension_info: argm.get_flag("extension-info"),
    };

    if args.is_debug {
        println!("Path to file: {:?}", &args.file_path);
    }

    if args.gui {
        let app = app::App::default().with_scheme(app::Scheme::Oxy);
        app::get_system_colors();
    
        let mut buffer = text::TextBuffer::default();
        buffer.set_tab_distance(4);
    
        let state = State::new(buffer.clone());
        app::GlobalState::new(state);
    
        let mut window = window::Window::default()
            .with_size(WIDTH, HEIGHT)
            .with_label("fat 0.3.0");
        window.set_xclass("fat");
        {
            let mut col = group::Flex::default_fill().column();
            col.set_pad(0);
            let mut m = menu::SysMenuBar::default();
            init_menu(&mut m);
            let mut ed = text::TextEditor::default().with_id("ed");
            ed.set_buffer(buffer);
            ed.set_linenumber_width(40);
            ed.set_text_font(Font::Courier);
            ed.set_trigger(CallbackTrigger::Changed);
            ed.set_callback(editor_cb);
            handle_drag_drop(&mut ed);
            window.resizable(&col);
            col.fixed(&m, 30);
            col.end();
        }
        window.end();
        window.show();
        window.set_callback(win_cb);
        app.run().unwrap();
    } else {
        get_info(&args);
    }
}
