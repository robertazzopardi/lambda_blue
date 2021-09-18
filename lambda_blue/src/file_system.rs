use native_dialog::FileDialog;
use std::{env, fs, path::PathBuf, thread};

use crate::emulator::{Emulator, Emulators};

pub fn get_execution_dir() -> PathBuf {
    let mut path_buf = env::current_exe().unwrap();
    path_buf.pop();

    path_buf
}

pub fn append_to_exec_dir(to_append: &str) -> PathBuf {
    let mut new_path = get_execution_dir();
    new_path.push(to_append);

    new_path
}

pub fn choose_file_dialog() -> Result<String, String> {
    let file_dialog = FileDialog::new();
    let path = file_dialog
        .set_location(home::home_dir().unwrap().to_str().unwrap())
        .show_open_single_file()
        .unwrap();

    let path = match path {
        Some(it) => it,
        None => return Err("Something went wrong choosing a rom file!".to_string()),
    };

    Ok(path.to_str().unwrap().to_string().replace("file://", ""))
}

pub fn save_emulators(emulators: &[Emulator]) {
    #[cfg(debug_assertions)]
    let json_string = serde_json::to_string_pretty(emulators).unwrap();

    #[cfg(not(debug_assertions))]
    let json_string = serde_json::to_string(emulators).unwrap();

    let save_path = append_to_exec_dir("emulators.json");

    let save_thread = thread::spawn(|| {
        fs::write(save_path, json_string).expect("Could not save emulator configuration!");
    });

    save_thread.join().expect("Thread error")
}

pub fn load_emulators() -> Result<Emulators, String> {
    let load_path = append_to_exec_dir("emulators.json");

    let read_emulators = fs::read_to_string(load_path);

    if let Ok(read_emulators) = read_emulators {
        if read_emulators.is_empty() {
            return Err("Emulator file empty".to_string());
        }

        let data: Vec<Emulator> = serde_json::from_str(&read_emulators).unwrap();

        Ok(Emulators { emulators: data })
    } else {
        Err("Could not read emulator file".to_string())
    }
}
