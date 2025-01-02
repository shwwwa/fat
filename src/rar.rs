use crate::Arguments;
use unrar::{ListSplit, VolumeInfo};

/// Gets specified rar info about file.
pub fn get_rar_info(args: &Arguments) {
    println!("## RAR information");
    let mut option = None;
    match unrar::Archive::new(&args.file_path).break_open::<ListSplit>(Some(&mut option)) {
        // Looks like I need to write my own implementations of rar lib
        Ok(archive) => {
            if archive.has_comment() {
                println!("# Comment: currently not supported",)
            }
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
