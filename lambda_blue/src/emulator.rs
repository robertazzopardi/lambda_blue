use crate::{file_system, EXIT_TEXT};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

pub enum LoadedEmulator {
    Yes(Emulator),
    No,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Rom {
    name: String,
    path: PathBuf,
}

impl Rom {
    pub fn new(path: &Path) -> Self {
        Self {
            name: path.file_stem().unwrap().to_str().unwrap().to_string(),
            path: path.to_path_buf(),
        }
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Emulators {
    emulators: Vec<Emulator>,
}

impl Emulators {
    pub fn new_from_vec(emulators: Vec<Emulator>) -> Self {
        Self { emulators }
    }

    pub fn from_strings(place_holders: Vec<&str>) -> Self {
        Emulators {
            emulators: place_holders
                .iter()
                .map(|s| Emulator::new(PathBuf::from_str(s).unwrap()))
                .collect(),
        }
    }

    // pub fn load_roms_for_emulator(&self, index: usize) -> Vec<&str> {
    //     self.emulators[index]
    //         .roms()
    //         .iter()
    //         .map(|r| r.name())
    //         .collect::<Vec<&str>>()
    // }

    pub fn load_rom_into_emulator(&mut self, rom: Rom, index: usize) {
        self.emulators[index].load_rom(rom)
    }

    pub fn get_emulator_clone(&self, index: usize) -> Emulator {
        self.emulators[index].clone()
    }

    /// Get a reference to the emulators's emulators.
    pub fn emulators(&self) -> &[Emulator] {
        self.emulators.as_slice()
    }

    pub fn emulator_names_to_string(&self) -> Vec<&str> {
        let mut main_page_texts = self
            .emulators
            .iter()
            .map(|emus| emus.name())
            .collect::<Vec<&str>>();

        main_page_texts.push(EXIT_TEXT);

        main_page_texts
    }

    pub fn emulator_index(&self, emulator: &Emulator) -> usize {
        self.emulators.iter().position(|em| em == emulator).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Emulator {
    name: String,
    path: PathBuf,
    roms: Vec<Rom>,

    #[serde(skip_serializing, skip_deserializing)]
    loaded_rom: Option<Rom>,
}

impl Emulator {
    pub fn new(path: PathBuf) -> Self {
        let name = path.file_stem().unwrap().to_str().unwrap();
        let emulator_path = file_system::append_to_exec_dir(name);

        Self {
            name: name.to_string(),
            path: emulator_path,
            roms: Vec::new(),
            loaded_rom: None,
        }
    }

    pub fn load_rom(&mut self, rom: Rom) {
        if !self.roms.contains(&rom) {
            self.roms.push(rom.clone());
        }

        self.loaded_rom = Some(rom)
    }

    pub fn get_rom_path(&self) -> &str {
        self.loaded_rom
            .as_ref()
            .unwrap()
            .get_path()
            .to_str()
            .unwrap()
    }

    pub fn get_path(&self) -> &str {
        self.path.to_str().unwrap()
    }

    /// Get a reference to the emulator's name.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Get a reference to the emulator's roms.
    // pub fn roms(&self) -> &[Rom] {
    //     self.roms.as_slice()
    // }

    pub fn rom_at_index(&self, index: usize) -> Rom {
        self.roms[index].clone()
    }

    pub fn roms_as_str(&self) -> Vec<&str> {
        self.roms
            .iter()
            .map(|r| r.name.as_str())
            .collect::<Vec<&str>>()
    }
}
