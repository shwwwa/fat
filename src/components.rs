use serde::{Deserialize, Deserializer};
use serde_derive::Deserialize;
use strum_macros::{EnumString, IntoStaticStr};
use std::{str::FromStr, path::PathBuf};

/// The difference with file-format lib is that we need as much accurate representation of types as possible,
/// whereas in file_format categories used for quick choice of formats needed for application (why do you need other file's backups for regular app?).
#[derive(Debug, PartialEq, EnumString, IntoStaticStr)]
#[strum(ascii_case_insensitive)]
pub enum Category {
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
pub struct Extension {
    #[allow(dead_code)]
    pub id: String,
    pub extension: String,
    pub name: String,
    pub category: Category,
    pub description: String,
    pub further_reading: String,
    pub preferred_mime: String,
    pub mime: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct ExtensionVec {
    pub extensions: Vec<Extension>,
}

pub struct Arguments {
    pub file_path: PathBuf,
    pub extensions_path: PathBuf,
    pub is_debug: bool,
    pub is_human: bool,
    pub only_general: bool,
    pub ignore_general: bool,
    pub extension_info: bool,
}
