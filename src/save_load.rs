use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct SaveLoadPlugin;

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_save_data);
    }
}

/// The player's persistent save data.
///
/// This is what gets written to disk (or localStorage on web) and loaded back
/// when the game starts. It represents the player's progress across sessions.
///
/// ## How serde works here
///
/// #[derive(Serialize, Deserialize)] generates code at compile time that knows
/// how to convert this struct to/from any format serde supports (RON, JSON, etc).
/// You write ONE struct and get every format for free — you just pick which
/// serializer to call. We use RON, but switching to JSON would be a one-line change.
///
/// ## Adding new fields
///
/// When you add a new field (like `gold: u32`), old save files won't have it.
/// Use #[serde(default)] on the field so serde fills in a default value instead
/// of erroring. This is how you handle save file migration without writing
/// manual migration code.
#[derive(Resource, Serialize, Deserialize, Debug, Clone)]
pub struct SaveData {
    /// How many of each slime type the player owns.
    /// #[serde(default)] on each field means old save files that only had
    /// `slime_count` won't fail to parse — missing fields get their Default
    /// value (0 for u32) instead of a deserialization error.
    #[serde(default)]
    pub normal_slimes: u32,
    #[serde(default)]
    pub tanks: u32,
    #[serde(default)]
    pub wizards: u32,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            normal_slimes: 5,
            tanks: 0,
            wizards: 0,
        }
    }
}

// =============================================================================
// Storage backend: Native (macOS, Linux, Windows)
//
// #[cfg(...)] is Rust's conditional compilation. The compiler completely excludes
// code that doesn't match the current target — it's not an if-statement at
// runtime, the code literally doesn't exist in the binary. Zero cost.
//
// For native platforms, we save to the filesystem using the `dirs` crate to
// find the right directory for each OS.
// =============================================================================

#[cfg(not(target_arch = "wasm32"))]
mod storage {
    use super::SaveData;
    use bevy::prelude::*;

    /// Returns the path to the save file in the project root (save.ron).
    fn save_file_path() -> Option<std::path::PathBuf> {
        Some(std::path::PathBuf::from("save.ron"))
    }

    /// Reads SaveData from the filesystem, or returns None if no save exists.
    pub fn load() -> Option<SaveData> {
        let path = save_file_path()?;

        if !path.exists() {
            info!("No save file found at {:?}. Starting fresh.", path);
            return None;
        }

        match std::fs::read_to_string(&path) {
            Ok(contents) => match ron::from_str::<SaveData>(&contents) {
                Ok(data) => {
                    info!("Loaded save data from {:?}: {:?}", path, data);
                    Some(data)
                }
                Err(e) => {
                    // File exists but is corrupted or has an outdated format.
                    // Log the error and fall back to defaults rather than crashing.
                    error!("Failed to parse save file: {}. Using defaults.", e);
                    None
                }
            },
            Err(e) => {
                error!("Failed to read save file: {}. Using defaults.", e);
                None
            }
        }
    }

    /// Writes SaveData to the filesystem.
    pub fn save(save_data: &SaveData) {
        let Some(path) = save_file_path() else {
            error!("Could not determine save file path!");
            return;
        };

        // Create the directory if it doesn't exist.
        // create_dir_all is like `mkdir -p` — creates all parent dirs too.
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                error!("Failed to create save directory: {}", e);
                return;
            }
        }

        // PrettyConfig makes the output human-readable (indented, one field per line).
        // Without it, everything would be on one line.
        let pretty = ron::ser::PrettyConfig::default();
        match ron::ser::to_string_pretty(save_data, pretty) {
            Ok(serialized) => {
                if let Err(e) = std::fs::write(&path, serialized) {
                    error!("Failed to write save file: {}", e);
                } else {
                    info!("Game saved to {:?}", path);
                }
            }
            Err(e) => error!("Failed to serialize save data: {}", e),
        }
    }
}

// =============================================================================
// Storage backend: WASM (browser)
//
// When you eventually compile for the web with `cargo build --target wasm32-unknown-unknown`,
// this block will be used instead of the native one above.
//
// Browsers don't have a filesystem, but they do have localStorage — a simple
// key-value store that persists across page reloads. Perfect for save data.
//
// You'll need to add `web-sys` or `gloo-storage` as a dependency when you
// get to this point. For now, this is a placeholder showing the structure.
// =============================================================================

#[cfg(target_arch = "wasm32")]
mod storage {
    use super::SaveData;
    use bevy::prelude::*;

    pub fn load() -> Option<SaveData> {
        // TODO: When targeting WASM, implement using web-sys or gloo-storage:
        //
        // let window = web_sys::window()?;
        // let local_storage = window.local_storage().ok()??;
        // let data_str = local_storage.get_item("gamble-game-2-save").ok()??;
        // ron::from_str::<SaveData>(&data_str).ok()
        //
        warn!("WASM save/load not yet implemented. Using defaults.");
        None
    }

    pub fn save(save_data: &SaveData) {
        // TODO: When targeting WASM, implement using web-sys or gloo-storage:
        //
        // let window = web_sys::window().expect("no window");
        // let local_storage = window.local_storage().unwrap().expect("no localStorage");
        // let pretty = ron::ser::PrettyConfig::default();
        // let serialized = ron::ser::to_string_pretty(save_data, pretty).unwrap();
        // local_storage.set_item("gamble-game-2-save", &serialized).unwrap();
        //
        warn!("WASM save/load not yet implemented.");
    }
}

// =============================================================================
// Public API — these are what the rest of the game calls.
// They delegate to whichever storage backend was compiled in.
// =============================================================================

/// Startup system: loads the save file, or creates default save data if none exists.
///
/// Inserts SaveData as a Bevy Resource so any system can access it via
/// Res<SaveData> (read-only) or ResMut<SaveData> (read-write).
fn load_save_data(mut commands: Commands) {
    let save_data = storage::load().unwrap_or_default();
    commands.insert_resource(save_data);
}

/// Saves the current SaveData to disk (or localStorage on web).
///
/// This is a plain function, not a system. Call it from systems at specific
/// moments (after a battle, when the player quits, etc.) rather than every frame.
///
/// Example usage from a system:
/// ```rust
/// fn end_of_battle(save_data: Res<SaveData>) {
///     save_to_disk(&save_data);
/// }
/// ```
pub fn save_to_disk(save_data: &SaveData) {
    storage::save(save_data);
}
