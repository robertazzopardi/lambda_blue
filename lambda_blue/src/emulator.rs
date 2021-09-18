use crate::EXIT_TEXT;
use serde::{Deserialize, Serialize};

pub enum LoadedEmulator {
    Yes(Emulator),
    No,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Rom {
    name: String,
}

impl Rom {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    /// Get a reference to the rom's name.
    pub fn name(&self) -> &str {
        self.name.as_str()
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
                .map(|s| Emulator::new(s.to_string()))
                .collect(),
        }
    }

    pub fn load_roms_for_emulator(&self, index: usize) -> Vec<&str> {
        self.emulators[index]
            .roms()
            .iter()
            .map(|r| r.name())
            .collect::<Vec<&str>>()
    }

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
    roms: Vec<Rom>,

    #[serde(skip_serializing, skip_deserializing)]
    loaded_rom: Option<Rom>,
}

impl Emulator {
    pub fn new(name: String) -> Self {
        Self {
            name,
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

    /// Get a reference to the emulator's loaded rom.
    pub const fn loaded_rom(&self) -> Option<&Rom> {
        self.loaded_rom.as_ref()
    }

    /// Get a reference to the emulator's name.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Get a reference to the emulator's roms.
    pub fn roms(&self) -> &[Rom] {
        self.roms.as_slice()
    }

    pub fn roms_as_str(&self) -> Vec<&str> {
        self.roms
            .iter()
            .map(|r| r.name.as_str())
            .collect::<Vec<&str>>()
    }
}
