use std::{fs::File, io::{BufRead, BufReader}, path::PathBuf};

use fltk::{app, button::Button, frame::Frame, prelude::*, window::Window};

fn is_empty_and_readable(path: &PathBuf) -> bool {
    path.read_dir().map(|mut i| i.next().is_none()).unwrap_or(false)
}
fn main() {
    let app = app::App::default();
    let mut wind = Window::new(100, 100, 400, 300, "ETS2 save editor v0.1.1");
    let mut frame = Frame::new(0, 0, 400, 200, "");
    let mut but = Button::new(160, 210, 80, 40, "Click me!");
    wind.end();
    wind.show();
    but.set_callback(move |_| frame.set_label("Hello World!")); // the closure capture is mutable borrow to our button
    
    let mut path_to_profiles : PathBuf = Default::default();
    // Start of ETS2 save editor.
    // For now supports only windows
    match dirs_next::document_dir() {
        Some(path) => {
            path.clone_into(&mut path_to_profiles);
        },
        None => {
            println!("Couldn't find a documents folder in your system. Exiting app...");
            // TODO: proper handling for app.quit()
            app.quit();
        }
    }
    
    path_to_profiles.push("Euro Truck Simulator 2");
    path_to_profiles.push("profiles");

    if !path_to_profiles.exists() {
        println!("Path to profiles does not exist/is not accessible. Please launch ETS 2 at least once.");
        app.quit();
    }

    // Checking if directory readable & empty
    if is_empty_and_readable(&path_to_profiles) {
        println!("Couldn't find any profiles, although profile folder exists. Trying to check steam cloud saves...");
        
        // TODO: properly 
        path_to_profiles = PathBuf::from(r"C:\Program Files (x86)\Steam\userdata\362100980\227300\remote\profiles\");
        // TODO: errors if unreadable, need a proper way to handle
        if is_empty_and_readable(&path_to_profiles) {
            println!("Path to profiles does not exist/is not accessible. Please launch ETS 2 at least once.");
            app.quit();
        }
    }
    // My magic profile name
    path_to_profiles.push(r"D181616666696465765F6D6F62696C65");
    // My magically decrypted profile.sii
    path_to_profiles.push("profile.sii2");
    let file = File::open(path_to_profiles).unwrap();
    let buf_reader = BufReader::new(file);
    let mut lines = buf_reader.lines();
    let next_line = lines.next().unwrap().unwrap();
    if next_line == "SiiNunit" {
        println!("File provided is correct");
    }
    app.run().unwrap();


}