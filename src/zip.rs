use crate::{get_extension_from_id, get_extension_name, Arguments};
use bytesize::ByteSize;
use std::io::{BufReader, Error};
use std::{ffi::OsStr, fs, fs::File};
use zip::CompressionMethod;
use zip::DateTime;

/// Is zip file is just a wrapper for other file format.
/// If true, returns id of extension. If false, returns "zip" id.
/// Does not return error, but can return in future.
pub fn get_complex_zip_id(buf_reader: BufReader<File>) -> Result<String, Error> {
    let mut archive = zip::ZipArchive::new(buf_reader).unwrap();

    // Jar and ear have both MANIFEST.mf file, if it has also application.xml then it's ear, if it does not it's jar
    let mut jar_ear_situation: bool = false;
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
            "META-INF/MANIFEST.MF" => {
                jar_ear_situation = true;
            }
            "META-INF/mozilla.rsa" => return Ok("xpi".to_string()),
            "WEB-INF/web.xml" => return Ok("war".to_string()),
            "doc.kml" => return Ok("kmz".to_string()),
            "document.json" => return Ok("sketch43".to_string()),
            "extension.vsixmanifest" => return Ok("vsix".to_string()),
            _ => {
                if file.name().starts_with("Fusion[Active]/") {
                    return Ok("autodesk123d".to_string());
                } else if file.name().starts_with("circuitdiagram/") {
                    return Ok("cddx".to_string());
                } else if file.name().starts_with("dwf/") {
                    return Ok("dwfx".to_string());
                } else if file.name().ends_with(".fb2") && !file.name().contains('/') {
                    return Ok("fbz".to_string());
                } else if file.name().starts_with("FusionAssetName[Active]/") {
                    return Ok("fusion360".to_string());
                } else if file.name().starts_with("Payload/") && file.name().contains(".app/") {
                    return Ok("ipa".to_string());
                } else if file.name().starts_with("word/") {
                    return Ok("ooxmldocument".to_string());
                } else if file.name().starts_with("visio/") {
                    return Ok("ooxmldrawing".to_string());
                } else if file.name().starts_with("ppt/") {
                    return Ok("ooxmlpresentation".to_string());
                } else if file.name().starts_with("xl/") {
                    return Ok("ooxmlspreadsheet".to_string());
                } else if file.name().starts_with("Documents/") && file.name().ends_with(".fpage") {
                    return Ok("xps".to_string());
                } else if file.name().starts_with("SpaceClaim/") {
                    return Ok("scdoc".to_string());
                } else if file.name().starts_with("3D/") && file.name().ends_with(".model") {
                    return Ok("3mf".to_string());
                } else if (file.name().ends_with(".usd")
                    || file.name().ends_with(".usda")
                    || file.name().ends_with(".usdc"))
                    && !file.name().contains('/')
                {
                    return Ok("usdz".to_string());
                }
            }
        };
    }
    if jar_ear_situation {
        return Ok("jar".to_string());
    }
    // Nothing was found, return regular zip file
    Ok("zip".to_string())
}

/// Is zip file is just a wrapper for other file format.
/// If true, returns extension string. If false, returns "zip" extension.
/// If scanning gives error, returns error.
pub fn get_complex_zip_extension(
    args: &Arguments,
    buf_reader: BufReader<File>,
) -> Result<String, Error> {
    match get_complex_zip_id(buf_reader) {
        Ok(id) => get_extension_from_id(args, id),
        Err(e) => Err(e),
    }
}

/// Gets specified zip info about file.
pub fn get_zip_info(args: &Arguments, buf_reader: BufReader<File>) {
    println!("## ZIP information");
    let mut archive = zip::ZipArchive::new(buf_reader).unwrap();
    if !archive.comment().is_empty() {
        println!(
            "# Comment: {:?}",
            std::str::from_utf8(archive.comment()).unwrap()
        );
    }

    print!("# Compressed size: ");
    let size: u64 = fs::metadata(args.file_path.clone()).unwrap().len();
    let decompressed_size: u64 = archive.decompressed_size().unwrap_or(0).try_into().unwrap();
    let mut percent = (size as f32 / decompressed_size as f32) * 100.;
    if percent > 100. {
        percent = 100.
    }
    if args.is_human {
        println!(
            "{}/{} ({:.2}%)",
            ByteSize(size).to_string_as(true),
            ByteSize(decompressed_size).to_string_as(true),
            percent
        )
    } else {
        println!("{}/{} ({:.2}%)", size, decompressed_size, percent);
    }

    // While we gather zip file information, gather also used compression methods
    let mut compression_methods: Vec<CompressionMethod> = Vec::new();
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
            println!("\"{}\"", outpath.display());
        } else {
            let last_modified: DateTime = file.last_modified().unwrap_or_default();
            let percent = format!(
                "{:.prec$}",
                ((file.compressed_size() as f32 / file.size() as f32) * 100.).min(100.),
                prec = 2
            )
            .to_string();
            let file_size: String = if args.is_human {
                ByteSize(file.compressed_size()).to_string_as(true)
                    + "/"
                    + &ByteSize(file.size()).to_string_as(true)
                    + ") ("
                    + &percent
                    + "%"
            } else {
                file.compressed_size().to_string()
                    + "/"
                    + &file.size().to_string()
                    + ") ("
                    + &percent
                    + "%"
            };
            print!(
                "\"{}\" ({}) ({}) (last modified: {}) ({})",
                outpath.display(),
                file_size,
                get_extension_name(
                    args,
                    file.mangled_name().extension().unwrap_or(OsStr::new(""))
                ),
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
