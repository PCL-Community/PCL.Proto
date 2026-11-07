//! Minecraft world metadata and CE-compatible settings in level.dat.
use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};

use fastnbt::Value;
use flate2::{
    Compression,
    read::{GzDecoder, ZlibDecoder},
    write::GzEncoder,
};
use serde::{Deserialize, Serialize};

const MAX_NBT_BYTES: u64 = 64 * 1024 * 1024;
const MODERN_DIFFICULTY: i64 = 4774;
static WRITE_LOCK: Mutex<()> = Mutex::new(());
type Compound = HashMap<String, Value>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveInfo {
    pub level_name: String,
    pub version_name: Option<String>,
    pub version_id: Option<i64>,
    pub seed: Option<String>,
    pub last_played: i64,
    pub spawn: Option<[i32; 3]>,
    pub game_mode: i64,
    pub difficulty: Option<u8>,
    pub difficulty_locked: bool,
    pub hardcore: bool,
    pub allow_commands: bool,
    pub can_edit_commands: bool,
    pub can_edit_difficulty: bool,
    pub can_edit_lock: bool,
    pub play_ticks: i64,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveChanges {
    pub allow_commands: Option<bool>,
    pub difficulty: Option<u8>,
    pub difficulty_locked: Option<bool>,
}

fn compound<'a>(data: &'a Compound, key: &str) -> Option<&'a Compound> {
    match data.get(key) {
        Some(Value::Compound(value)) => Some(value),
        _ => None,
    }
}

fn number(data: &Compound, key: &str) -> Option<i64> {
    match data.get(key) {
        Some(Value::Byte(v)) => Some(i64::from(*v)),
        Some(Value::Short(v)) => Some(i64::from(*v)),
        Some(Value::Int(v)) => Some(i64::from(*v)),
        Some(Value::Long(v)) => Some(*v),
        _ => None,
    }
}

fn string<'a>(data: &'a Compound, key: &str) -> Option<&'a str> {
    data.get(key).and_then(Value::as_str)
}

fn read_file(path: &Path) -> Result<(Vec<u8>, Vec<u8>), String> {
    let metadata = fs::symlink_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if !metadata.is_file() || metadata.len() > MAX_NBT_BYTES {
        return Err(format!("Invalid or oversized NBT file: {}", path.display()));
    }
    let original = fs::read(path).map_err(|e| e.to_string())?;
    let reader: Box<dyn Read> = if original.starts_with(&[0x1f, 0x8b]) {
        Box::new(GzDecoder::new(original.as_slice()))
    } else if original.len() >= 2
        && original[0] & 0x0f == 8
        && u16::from_be_bytes([original[0], original[1]]) % 31 == 0
    {
        Box::new(ZlibDecoder::new(original.as_slice()))
    } else {
        Box::new(original.as_slice())
    };
    let mut decoded = Vec::new();
    reader
        .take(MAX_NBT_BYTES + 1)
        .read_to_end(&mut decoded)
        .map_err(|e| e.to_string())?;
    if decoded.len() as u64 > MAX_NBT_BYTES {
        return Err("NBT file is too large".into());
    }
    Ok((original, decoded))
}

fn read_nbt(path: &Path) -> Result<(Vec<u8>, Vec<u8>, Compound), String> {
    let (original, decoded) = read_file(path)?;
    let root: Compound = fastnbt::from_bytes(&decoded)
        .map_err(|e| format!("Invalid NBT in {}: {e}", path.display()))?;
    Ok((original, decoded, root))
}

fn source_path(folder: &Path) -> Result<PathBuf, String> {
    let path = folder.join("level.dat");
    match fs::symlink_metadata(&path) {
        Ok(_) => Ok(path),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(folder.join("level.dat_old")),
        Err(e) => Err(e.to_string()),
    }
}

fn save_data(root: &Compound) -> Result<&Compound, String> {
    compound(root, "Data").ok_or_else(|| "Missing Data compound in level.dat".into())
}

fn info_from_data(folder: &Path, data: &Compound) -> SaveInfo {
    let version = compound(data, "Version");
    let settings = compound(data, "difficulty_settings");
    let modern =
        number(data, "DataVersion").is_some_and(|v| v >= MODERN_DIFFICULTY) || settings.is_some();
    let seed = compound(data, "WorldGenSettings")
        .and_then(|v| number(v, "seed"))
        .or_else(|| number(data, "RandomSeed"))
        .or_else(|| {
            if !modern {
                return None;
            }
            let file = folder.join("data/minecraft/world_gen_settings.dat");
            let (_, _, root) = read_nbt(&file).ok()?;
            number(compound(&root, "data")?, "seed")
        });
    let spawn = compound(data, "spawn")
        .and_then(|v| match v.get("pos") {
            Some(Value::IntArray(pos)) if pos.len() == 3 => Some([pos[0], pos[1], pos[2]]),
            _ => None,
        })
        .or_else(|| {
            Some([
                i32::try_from(number(data, "SpawnX")?).ok()?,
                i32::try_from(number(data, "SpawnY")?).ok()?,
                i32::try_from(number(data, "SpawnZ")?).ok()?,
            ])
        });
    let difficulty = settings
        .and_then(|v| string(v, "difficulty"))
        .and_then(|v| match v {
            "peaceful" => Some(0),
            "easy" => Some(1),
            "normal" => Some(2),
            "hard" => Some(3),
            _ => None,
        })
        .or_else(|| {
            number(data, "Difficulty")
                .and_then(|n| u8::try_from(n).ok())
                .filter(|n| *n <= 3)
        });
    let hardcore = settings
        .and_then(|v| number(v, "hardcore"))
        .or_else(|| number(data, "hardcore"))
        .unwrap_or(0)
        == 1;
    let locked = settings
        .and_then(|v| number(v, "locked"))
        .or_else(|| number(data, "DifficultyLocked"))
        .unwrap_or(0)
        == 1;
    SaveInfo {
        level_name: string(data, "LevelName").unwrap_or("unknown").to_owned(),
        version_name: version.and_then(|v| string(v, "Name")).map(str::to_owned),
        version_id: version.and_then(|v| number(v, "Id")),
        seed: seed.map(|v| v.to_string()),
        last_played: number(data, "LastPlayed").unwrap_or(0),
        spawn,
        game_mode: if hardcore {
            4
        } else {
            number(data, "GameType")
                .filter(|v| (0..=3).contains(v))
                .unwrap_or(0)
        },
        difficulty,
        difficulty_locked: locked,
        hardcore,
        allow_commands: number(data, "allowCommands").unwrap_or(0) == 1,
        can_edit_commands: matches!(data.get("allowCommands"), Some(Value::Byte(_))),
        can_edit_difficulty: modern || matches!(data.get("Difficulty"), Some(Value::Byte(_))),
        can_edit_lock: modern || matches!(data.get("DifficultyLocked"), Some(Value::Byte(_))),
        play_ticks: number(data, "Time").unwrap_or(0),
    }
}

pub fn read(folder: &Path) -> Result<SaveInfo, String> {
    let (_, _, root) = read_nbt(&source_path(folder)?)?;
    Ok(info_from_data(folder, save_data(&root)?))
}

pub fn apply(folder: &Path, changes: SaveChanges) -> Result<SaveInfo, String> {
    let _guard = WRITE_LOCK.lock().map_err(|e| e.to_string())?;
    if changes.difficulty.is_some_and(|v| v > 3) {
        return Err("Invalid difficulty".into());
    }
    let source = source_path(folder)?;
    let (original, decoded, mut root) = read_nbt(&source)?;
    // fastnbt drops the element type of empty lists when serializing them.
    let mut parser = fastnbt::stream::Parser::new(decoded.as_slice());
    let mut depth = 0usize;
    loop {
        match parser.next().map_err(|e| e.to_string())? {
            fastnbt::stream::Value::List(_, tag, 0) if tag != fastnbt::Tag::End => {
                return Err("Cannot safely edit an NBT file containing typed empty lists".into());
            }
            fastnbt::stream::Value::Compound(_) | fastnbt::stream::Value::List(_, _, _) => {
                depth += 1
            }
            fastnbt::stream::Value::CompoundEnd | fastnbt::stream::Value::ListEnd => {
                depth = depth.checked_sub(1).ok_or("Invalid NBT structure")?;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
    }
    let data = match root.get_mut("Data") {
        Some(Value::Compound(data)) => data,
        _ => return Err("Missing Data compound in level.dat".into()),
    };
    let current = info_from_data(folder, data);
    if (changes.allow_commands.is_some() && !current.can_edit_commands)
        || (changes.difficulty.is_some() && !current.can_edit_difficulty)
        || (changes.difficulty_locked.is_some() && (!current.can_edit_lock || current.hardcore))
    {
        return Err("This save does not support that change".into());
    }
    if let Some(value) = changes.allow_commands {
        data.insert("allowCommands".into(), Value::Byte(value.into()));
    }
    if number(data, "DataVersion").is_some_and(|v| v >= MODERN_DIFFICULTY)
        || compound(data, "difficulty_settings").is_some()
    {
        if changes.difficulty.is_some() || changes.difficulty_locked.is_some() {
            let settings = data
                .entry("difficulty_settings".to_owned())
                .or_insert_with(|| Value::Compound(Compound::new()));
            let Value::Compound(settings) = settings else {
                return Err("Invalid difficulty_settings".into());
            };
            if let Some(value) = changes.difficulty {
                settings.insert(
                    "difficulty".into(),
                    Value::String(
                        ["peaceful", "easy", "normal", "hard"][usize::from(value)].into(),
                    ),
                );
            }
            if let Some(value) = changes.difficulty_locked {
                settings.insert("locked".into(), Value::Byte(value.into()));
            }
        }
    } else {
        if let Some(value) = changes.difficulty {
            data.insert("Difficulty".into(), Value::Byte(value as i8));
        }
        if let Some(value) = changes.difficulty_locked {
            data.insert("DifficultyLocked".into(), Value::Byte(value.into()));
        }
    }
    if changes.allow_commands.is_none()
        && changes.difficulty.is_none()
        && changes.difficulty_locked.is_none()
    {
        return Ok(current);
    }

    // Preserve the root compound's original name; fastnbt otherwise emits an empty name.
    if decoded.len() < 3 || decoded[0] != 10 {
        return Err("Invalid NBT root".into());
    }
    let name_end = 3 + usize::from(u16::from_be_bytes([decoded[1], decoded[2]]));
    if name_end > decoded.len() {
        return Err("Invalid NBT root name".into());
    }
    let encoded = fastnbt::to_bytes(&root).map_err(|e| e.to_string())?;
    let mut staged = tempfile::NamedTempFile::new_in(folder).map_err(|e| e.to_string())?;
    {
        let mut encoder = GzEncoder::new(&mut staged, Compression::default());
        encoder
            .write_all(&decoded[..name_end])
            .and_then(|_| encoder.write_all(&encoded[3..]))
            .map_err(|e| e.to_string())?;
        encoder.finish().map_err(|e| e.to_string())?;
    }
    staged.as_file().sync_all().map_err(|e| e.to_string())?;
    if fs::read(&source).map_err(|e| e.to_string())? != original {
        return Err("Save changed while editing; reload and try again".into());
    }
    if source == folder.join("level.dat") {
        let mut backup = tempfile::NamedTempFile::new_in(folder).map_err(|e| e.to_string())?;
        backup.write_all(&original).map_err(|e| e.to_string())?;
        backup.as_file().sync_all().map_err(|e| e.to_string())?;
        backup
            .persist(folder.join("level.dat_old"))
            .map_err(|e| e.to_string())?;
    }
    staged
        .persist(folder.join("level.dat"))
        .map_err(|e| e.to_string())?;
    read(folder)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_save(dir: &Path, filename: &str, root: &Compound) -> Vec<u8> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(&fastnbt::to_bytes(root).unwrap())
            .unwrap();
        let bytes = encoder.finish().unwrap();
        fs::write(dir.join(filename), &bytes).unwrap();
        bytes
    }

    #[test]
    fn edit_preserves_unknown_fields_and_backs_up_original() {
        let dir = tempfile::tempdir().unwrap();
        let mut data = Compound::new();
        data.insert("LevelName".into(), Value::String("Test".into()));
        data.insert("RandomSeed".into(), Value::Long(i64::MIN));
        data.insert("allowCommands".into(), Value::Byte(0));
        data.insert("Difficulty".into(), Value::Byte(2));
        data.insert("unknown".into(), Value::Long(42));
        let root = Compound::from([("Data".into(), Value::Compound(data))]);
        let original = write_save(dir.path(), "level.dat", &root);
        let info = apply(
            dir.path(),
            SaveChanges {
                allow_commands: Some(true),
                difficulty: Some(3),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(info.seed.as_deref(), Some("-9223372036854775808"));
        assert!(info.allow_commands);
        assert_eq!(info.difficulty, Some(3));
        assert_eq!(
            fs::read(dir.path().join("level.dat_old")).unwrap(),
            original
        );
        let (_, _, result) = read_nbt(&dir.path().join("level.dat")).unwrap();
        assert_eq!(number(save_data(&result).unwrap(), "unknown"), Some(42));
    }

    #[test]
    fn backup_only_is_not_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let data = Compound::from([("allowCommands".into(), Value::Byte(0))]);
        let root = Compound::from([("Data".into(), Value::Compound(data))]);
        let original = write_save(dir.path(), "level.dat_old", &root);
        apply(
            dir.path(),
            SaveChanges {
                allow_commands: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            fs::read(dir.path().join("level.dat_old")).unwrap(),
            original
        );
        assert!(read(dir.path()).unwrap().allow_commands);
    }

    #[test]
    fn typed_empty_lists_are_not_rewritten() {
        let dir = tempfile::tempdir().unwrap();
        let mut data = Compound::new();
        data.insert("allowCommands".into(), Value::Byte(0));
        data.insert("list".into(), Value::List(vec![]));
        let root = Compound::from([("Data".into(), Value::Compound(data))]);
        let mut bytes = fastnbt::to_bytes(&root).unwrap();
        let pos = bytes.windows(6).position(|w| w == b"\x00\x04list").unwrap() + 6;
        bytes[pos] = 3;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&bytes).unwrap();
        let original = encoder.finish().unwrap();
        fs::write(dir.path().join("level.dat"), &original).unwrap();
        assert!(
            apply(
                dir.path(),
                SaveChanges {
                    allow_commands: Some(true),
                    ..Default::default()
                }
            )
            .is_err()
        );
        assert_eq!(fs::read(dir.path().join("level.dat")).unwrap(), original);
        assert!(!dir.path().join("level.dat_old").exists());
    }
}
