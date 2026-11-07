//! Read and edit the server list without rewriting unrelated NBT tags.

use std::{
    fs,
    io::{Cursor, Read, Write},
    ops::Range,
    path::Path,
    sync::Mutex,
};

use fastnbt::{
    Tag,
    stream::{Parser, Value},
};
use flate2::{
    Compression,
    read::{GzDecoder, ZlibDecoder},
    write::{GzEncoder, ZlibEncoder},
};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

const LIMIT: u64 = 16 * 1024 * 1024;
const EMPTY: &[u8] = b"\x0a\0\0\0";
static WRITE_LOCK: Mutex<()> = Mutex::new(());

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerList {
    servers: Vec<Server>,
    revision: String,
}

#[derive(Serialize)]
pub struct Server {
    name: String,
    address: String,
    icon: Option<String>,
}

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub enum ServerChange {
    Add {
        name: String,
        address: String,
    },
    Edit {
        index: usize,
        name: String,
        address: String,
    },
    Remove {
        index: usize,
    },
}

struct Entry {
    range: Range<usize>,
    name_range: Option<Range<usize>>,
    address_range: Option<Range<usize>>,
    server: Server,
}

struct Document {
    entries: Vec<Entry>,
    list_header: Option<usize>,
    list_end: usize,
}

// Validate lengths before handing the data to fastnbt's allocating stream parser.
fn validate_nbt(bytes: &[u8]) -> Result<(), String> {
    fn take<'a>(bytes: &'a [u8], pos: &mut usize, n: usize) -> Result<&'a [u8], String> {
        let end = pos.checked_add(n).ok_or("NBT length overflow")?;
        let part = bytes.get(*pos..end).ok_or("Truncated NBT")?;
        *pos = end;
        Ok(part)
    }
    fn string(bytes: &[u8], pos: &mut usize) -> Result<(), String> {
        let n = take(bytes, pos, 2)?;
        let n = u16::from_be_bytes([n[0], n[1]]) as usize;
        take(bytes, pos, n)?;
        Ok(())
    }
    fn payload(bytes: &[u8], pos: &mut usize, tag: u8, depth: usize) -> Result<(), String> {
        if depth > 128 {
            return Err("NBT nesting too deep".into());
        }
        match tag {
            1..=6 => {
                take(bytes, pos, [0, 1, 2, 4, 8, 4, 8][tag as usize])?;
            }
            7 | 11 | 12 => {
                let raw: [u8; 4] = take(bytes, pos, 4)?.try_into().unwrap();
                let count =
                    usize::try_from(i32::from_be_bytes(raw)).map_err(|_| "Invalid array length")?;
                let width = match tag {
                    11 => 4,
                    12 => 8,
                    _ => 1,
                };
                take(
                    bytes,
                    pos,
                    count.checked_mul(width).ok_or("NBT length overflow")?,
                )?;
            }
            8 => string(bytes, pos)?,
            9 => {
                let element = take(bytes, pos, 1)?[0];
                let raw: [u8; 4] = take(bytes, pos, 4)?.try_into().unwrap();
                let count =
                    usize::try_from(i32::from_be_bytes(raw)).map_err(|_| "Invalid list length")?;
                if element > 12 || (element == 0 && count != 0) || count > bytes.len() {
                    return Err("Invalid list type or length".into());
                }
                for _ in 0..count {
                    payload(bytes, pos, element, depth + 1)?;
                }
            }
            10 => loop {
                let child = take(bytes, pos, 1)?[0];
                if child == 0 {
                    break;
                }
                string(bytes, pos)?;
                payload(bytes, pos, child, depth + 1)?;
            },
            _ => return Err("Invalid NBT tag".into()),
        }
        Ok(())
    }
    let mut pos = 0;
    if take(bytes, &mut pos, 1)?[0] != 10 {
        return Err("Invalid NBT root".into());
    }
    string(bytes, &mut pos)?;
    payload(bytes, &mut pos, 10, 0)?;
    if pos != bytes.len() {
        return Err("Trailing NBT data".into());
    }
    Ok(())
}

fn parse(bytes: &[u8]) -> Result<Document, String> {
    validate_nbt(bytes)?;
    let mut parser = Parser::new(Cursor::new(bytes));
    let mut depth = 0usize;
    let mut document = Document {
        entries: Vec::new(),
        list_header: None,
        list_end: 0,
    };
    let mut in_servers = false;
    let mut entry: Option<Entry> = None;
    loop {
        let start = parser.get_ref().position() as usize;
        let value = parser.next().map_err(|e| e.to_string())?;
        let end = parser.get_ref().position() as usize;
        let name = match &value {
            Value::Byte(name, _)
            | Value::Short(name, _)
            | Value::Int(name, _)
            | Value::Long(name, _)
            | Value::Float(name, _)
            | Value::Double(name, _)
            | Value::ByteArray(name, _)
            | Value::String(name, _)
            | Value::List(name, _, _)
            | Value::Compound(name)
            | Value::IntArray(name, _)
            | Value::LongArray(name, _) => name.as_deref(),
            _ => None,
        };
        if depth == 1 && name == Some("servers") && !matches!(value, Value::List(_, _, _)) {
            return Err("Invalid servers list type".into());
        }
        match value {
            Value::List(name, tag, count) if depth == 1 && name.as_deref() == Some("servers") => {
                if document.list_header.is_some()
                    || count < 0
                    || (tag != Tag::Compound && !(tag == Tag::End && count == 0))
                {
                    return Err("Invalid servers list".into());
                }
                document.list_header = Some(end - 5);
                in_servers = true;
                depth += 1;
            }
            Value::Compound(_) if in_servers && depth == 2 => {
                entry = Some(Entry {
                    range: start..start,
                    name_range: None,
                    address_range: None,
                    server: Server {
                        name: "Unknown".into(),
                        address: "Unknown".into(),
                        icon: None,
                    },
                });
                depth += 1;
            }
            Value::String(name, text) if in_servers && depth == 3 => {
                let entry = entry.as_mut().ok_or("Invalid server entry")?;
                match name.as_deref() {
                    Some("name") if entry.name_range.is_none() => {
                        entry.server.name = text;
                        entry.name_range = Some(start..end);
                    }
                    Some("ip") if entry.address_range.is_none() => {
                        entry.server.address = text;
                        entry.address_range = Some(start..end);
                    }
                    Some("icon") => entry.server.icon = Some(text),
                    Some("name" | "ip") => return Err("Duplicate server field".into()),
                    _ => {}
                }
            }
            Value::Byte(Some(ref name), _)
            | Value::Short(Some(ref name), _)
            | Value::Int(Some(ref name), _)
            | Value::Long(Some(ref name), _)
            | Value::Float(Some(ref name), _)
            | Value::Double(Some(ref name), _)
            | Value::ByteArray(Some(ref name), _)
            | Value::IntArray(Some(ref name), _)
            | Value::LongArray(Some(ref name), _)
            | Value::List(Some(ref name), _, _)
            | Value::Compound(Some(ref name))
                if in_servers && depth == 3 && (name == "name" || name == "ip") =>
            {
                return Err("Invalid server field type".into());
            }
            Value::Compound(_) | Value::List(_, _, _) => depth += 1,
            Value::CompoundEnd if in_servers && depth == 3 => {
                let mut completed = entry.take().ok_or("Invalid server entry")?;
                completed.range.end = end;
                document.entries.push(completed);
                depth -= 1;
            }
            Value::ListEnd if in_servers && depth == 2 => {
                document.list_end = end;
                in_servers = false;
                depth -= 1;
            }
            Value::CompoundEnd | Value::ListEnd => {
                depth = depth.checked_sub(1).ok_or("Invalid NBT structure")?;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
    }
    if document.list_header.is_none() {
        document.list_end = bytes.len() - 1;
    }
    Ok(document)
}

/// `(原始字节, 解码后字节, 压缩方式)`，压缩方式 0=无、1=gzip、2=zlib。
type ServersDatBytes = (Option<Vec<u8>>, Vec<u8>, u8);

fn read_file(path: &Path) -> Result<ServersDatBytes, String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok((None, EMPTY.to_vec(), 0)),
        Err(e) => return Err(e.to_string()),
    };
    if !metadata.is_file() || metadata.len() > LIMIT {
        return Err("Invalid or oversized servers.dat".into());
    }
    let mut original = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(LIMIT + 1)
        .read_to_end(&mut original)
        .map_err(|e| e.to_string())?;
    if original.len() as u64 > LIMIT {
        return Err("Oversized servers.dat".into());
    }
    let (reader, compression): (Box<dyn Read>, u8) = if original.starts_with(&[0x1f, 0x8b]) {
        (Box::new(GzDecoder::new(original.as_slice())), 1)
    } else if original.len() >= 2
        && original[0] & 15 == 8
        && u16::from_be_bytes([original[0], original[1]]) % 31 == 0
    {
        (Box::new(ZlibDecoder::new(original.as_slice())), 2)
    } else {
        (Box::new(original.as_slice()), 0)
    };
    let mut decoded = Vec::new();
    reader
        .take(LIMIT + 1)
        .read_to_end(&mut decoded)
        .map_err(|e| e.to_string())?;
    if decoded.len() as u64 > LIMIT {
        return Err("Oversized servers.dat".into());
    }
    Ok((Some(original), decoded, compression))
}

fn revision(original: Option<&[u8]>) -> String {
    original.map_or_else(
        || "missing".into(),
        |bytes| crate::io::files::sha1_hex(Sha1::digest(bytes)),
    )
}

fn field(name: &str, value: &str) -> Result<Vec<u8>, String> {
    let map = std::collections::HashMap::from([(name, value)]);
    let bytes = fastnbt::to_bytes(&map).map_err(|e| e.to_string())?;
    Ok(bytes[3..bytes.len() - 1].to_vec())
}

pub fn read(folder: &Path) -> Result<ServerList, String> {
    let (original, bytes, _) = read_file(&folder.join("servers.dat"))?;
    let document = parse(&bytes)?;
    Ok(ServerList {
        servers: document
            .entries
            .into_iter()
            .map(|entry| entry.server)
            .collect(),
        revision: revision(original.as_deref()),
    })
}

pub fn change(folder: &Path, expected: &str, change: ServerChange) -> Result<ServerList, String> {
    let _guard = WRITE_LOCK.lock().map_err(|e| e.to_string())?;
    let path = folder.join("servers.dat");
    let (original, mut bytes, compression) = read_file(&path)?;
    if revision(original.as_deref()) != expected {
        return Err("Server list changed; reload and try again".into());
    }
    let document = parse(&bytes)?;
    let (index, name, address, remove) = match change {
        ServerChange::Add { name, address } => (None, name, address, false),
        ServerChange::Edit {
            index,
            name,
            address,
        } => (Some(index), name, address, false),
        ServerChange::Remove { index } => (Some(index), String::new(), String::new(), true),
    };
    if !remove
        && (name.trim().is_empty()
            || address.trim().is_empty()
            || name.len() > 4096
            || address.len() > 4096)
    {
        return Err("Invalid server name or address".into());
    }
    let mut replacement = Vec::new();
    let range = if let Some(index) = index {
        let entry = document
            .entries
            .get(index)
            .ok_or("Server index out of date")?;
        if !remove {
            replacement.extend_from_slice(&bytes[entry.range.clone()]);
            let mut fields = vec![
                (entry.name_range.clone(), field("name", &name)?),
                (entry.address_range.clone(), field("ip", &address)?),
            ];
            fields.sort_by_key(|(range, _)| {
                std::cmp::Reverse(range.as_ref().map_or(entry.range.end - 1, |r| r.start))
            });
            for (range, value) in fields {
                let range = range.unwrap_or(entry.range.end - 1..entry.range.end - 1);
                replacement.splice(
                    range.start - entry.range.start..range.end - entry.range.start,
                    value,
                );
            }
        }
        entry.range.clone()
    } else {
        replacement.extend(field("name", &name)?);
        replacement.extend(field("ip", &address)?);
        replacement.push(0);
        document.list_end..document.list_end
    };
    if let Some(header) = document.list_header {
        let count = document.entries.len() + usize::from(index.is_none()) - usize::from(remove);
        bytes[header] = 10;
        bytes[header + 1..header + 5].copy_from_slice(&(count as i32).to_be_bytes());
    } else {
        if index.is_some() {
            return Err("Server index out of date".into());
        }
        let mut list = b"\x09\0\x07servers\x0a\0\0\0\x01".to_vec();
        list.extend(replacement);
        replacement = list;
    }
    bytes.splice(range, replacement);
    if bytes.len() as u64 > LIMIT {
        return Err("Oversized servers.dat".into());
    }
    parse(&bytes)?;
    fs::create_dir_all(folder).map_err(|e| e.to_string())?;
    let mut staged = tempfile::NamedTempFile::new_in(folder).map_err(|e| e.to_string())?;
    match compression {
        1 => {
            let mut out = GzEncoder::new(&mut staged, Compression::default());
            out.write_all(&bytes).map_err(|e| e.to_string())?;
            out.finish().map_err(|e| e.to_string())?;
        }
        2 => {
            let mut out = ZlibEncoder::new(&mut staged, Compression::default());
            out.write_all(&bytes).map_err(|e| e.to_string())?;
            out.finish().map_err(|e| e.to_string())?;
        }
        _ => staged.write_all(&bytes).map_err(|e| e.to_string())?,
    }
    staged.as_file().sync_all().map_err(|e| e.to_string())?;
    let (current, _, _) = read_file(&path)?;
    if current != original {
        return Err("Server list changed; reload and try again".into());
    }
    if original.is_some() {
        staged.persist(&path).map_err(|e| e.to_string())?;
    } else {
        staged.persist_noclobber(&path).map_err(|e| e.to_string())?;
    }
    read(folder)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_edits_and_removes_without_losing_unknown_data() {
        let folder = tempfile::tempdir().unwrap();
        let mut original = b"\x0a\0\x04root\x09\0\x07servers\x0a\0\0\0\x01".to_vec();
        original.extend(field("name", "Old").unwrap());
        original.extend(field("ip", "old.example").unwrap());
        original.extend_from_slice(b"\x09\0\x05extra\x03\0\0\0\0\0\x03\0\x05other\0\0\0\x2a\0");
        fs::write(folder.path().join("servers.dat"), &original).unwrap();
        let list = read(folder.path()).unwrap();
        assert_eq!(list.servers[0].name, "Old");
        let list = change(
            folder.path(),
            &list.revision,
            ServerChange::Edit {
                index: 0,
                name: "New 😃".into(),
                address: "new.example".into(),
            },
        )
        .unwrap();
        assert_eq!(list.servers[0].name, "New 😃");
        let modified = fs::read(folder.path().join("servers.dat")).unwrap();
        assert!(
            modified
                .windows(13)
                .any(|bytes| bytes == b"\x09\0\x05extra\x03\0\0\0\0")
        );
        assert!(
            modified
                .windows(12)
                .any(|bytes| bytes == b"\x03\0\x05other\0\0\0\x2a")
        );
        assert!(
            change(
                folder.path(),
                &revision(Some(&original)),
                ServerChange::Remove { index: 0 }
            )
            .is_err()
        );
        let list = change(
            folder.path(),
            &list.revision,
            ServerChange::Add {
                name: "Second".into(),
                address: "localhost".into(),
            },
        )
        .unwrap();
        assert_eq!(list.servers.len(), 2);
        let list = change(
            folder.path(),
            &list.revision,
            ServerChange::Remove { index: 0 },
        )
        .unwrap();
        assert_eq!(list.servers[0].name, "Second");
    }

    #[test]
    fn creates_missing_list_and_rejects_invalid_lengths() {
        let folder = tempfile::tempdir().unwrap();
        let list = read(folder.path()).unwrap();
        let list = change(
            folder.path(),
            &list.revision,
            ServerChange::Add {
                name: "Test".into(),
                address: "localhost".into(),
            },
        )
        .unwrap();
        assert_eq!(list.servers.len(), 1);
        let path = folder.path().join("servers.dat");
        let malformed = b"\x0a\0\0\x07\0\x01x\x7f\xff\xff\xff\0";
        fs::write(&path, malformed).unwrap();
        assert!(read(folder.path()).is_err());
        assert!(
            change(
                folder.path(),
                &list.revision,
                ServerChange::Remove { index: 0 }
            )
            .is_err()
        );
        assert_eq!(fs::read(path).unwrap(), malformed);
    }

    #[test]
    fn preserves_compression_and_rejects_wrong_list_type() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("servers.dat");
        let mut gzip = GzEncoder::new(Vec::new(), Compression::default());
        gzip.write_all(EMPTY).unwrap();
        fs::write(&path, gzip.finish().unwrap()).unwrap();
        let list = read(folder.path()).unwrap();
        change(
            folder.path(),
            &list.revision,
            ServerChange::Add {
                name: "A".into(),
                address: "localhost".into(),
            },
        )
        .unwrap();
        assert!(fs::read(&path).unwrap().starts_with(&[0x1f, 0x8b]));
        let invalid = b"\x0a\0\0\x08\0\x07servers\0\x01x\0";
        fs::write(&path, invalid).unwrap();
        assert!(read(folder.path()).is_err());
        assert_eq!(fs::read(path).unwrap(), invalid);
    }
}
