use crate::{hash::HashValue, utils::hash_directory};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    env::temp_dir,
    fs::{read_to_string, remove_file, write, File},
    io::{Read, Result, Write},
    path::{Path, PathBuf},
    time::SystemTime,
};
use zip::{write::FileOptions, ZipArchive, ZipWriter};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Change {
    Add,
    Update,
    Remove,
}

pub fn patch_request<P>(workspace: P, hashes: P) -> Result<()>
where
    P: AsRef<Path>,
{
    println!("* Patch request");
    let local_hashes = hash_directory(workspace.as_ref())?;
    println!("* Hashes: {:#?}", local_hashes);
    let data = serde_json::to_string_pretty(&local_hashes)?;
    write(hashes.as_ref(), data)
}

pub fn patch_create<P, PD>(workspace: P, hashes: P, archive: PD) -> Result<()>
where
    P: AsRef<Path>,
    PD: AsRef<Path> + std::fmt::Debug,
{
    println!("* Patch create");
    let local_hashes = hash_directory(workspace.as_ref())?;
    let hashes = read_to_string(hashes)?;
    let hashes = serde_json::from_str(&hashes)?;
    let changes = diff_changes(&hashes, &local_hashes);
    println!("* Changes: {:#?}", changes);
    let number = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos(),
        Err(_) => 0,
    };
    let mut archive_path = temp_dir();
    archive_path.push(format!("doctor-diff-{}.zip", number));
    archive_changes(workspace, archive, &changes)
}

pub fn patch_apply<P>(workspace: P, archive: P) -> Result<()>
where
    P: AsRef<Path>,
{
    unarchive_changes(workspace, archive)
}

pub fn diff_changes(
    client_hashes: &HashMap<PathBuf, HashValue>,
    server_hashes: &HashMap<PathBuf, HashValue>,
) -> HashMap<PathBuf, Change> {
    let mut result = HashMap::with_capacity(server_hashes.len());
    let server_paths = server_hashes.keys().collect::<HashSet<_>>();
    let client_paths = client_hashes.keys().collect::<HashSet<_>>();
    for path in server_paths.intersection(&client_paths) {
        let server_hash = server_hashes.get(*path).unwrap();
        let client_hash = client_hashes.get(*path).unwrap();
        if server_hash != client_hash {
            result.insert((*path).to_owned(), Change::Update);
        }
    }
    for path in server_paths.difference(&client_paths) {
        let server_hash = server_hashes.get(*path).unwrap();
        let client_hash = client_hashes.get(*path).unwrap();
        if server_hash != client_hash {
            result.insert((*path).to_owned(), Change::Add);
        }
    }
    for path in client_paths.intersection(&server_paths) {
        let server_hash = server_hashes.get(*path).unwrap();
        let client_hash = client_hashes.get(*path).unwrap();
        if server_hash != client_hash {
            result.insert((*path).to_owned(), Change::Remove);
        }
    }
    result
}

pub fn archive_changes<P, PD>(
    workspace: P,
    archive: PD,
    changes: &HashMap<PathBuf, Change>,
) -> Result<()>
where
    P: AsRef<Path>,
    PD: AsRef<Path> + std::fmt::Debug,
{
    println!("* Archive changes to: {:?}", archive.as_ref());
    let mut archive = ZipWriter::new(File::create(archive)?);
    let comment = serde_json::to_string_pretty(changes)?;
    archive.set_comment(&comment);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (path, change) in changes {
        match change {
            Change::Add | Change::Update => {
                println!("* Archive change: {:?}", path);
                let mut reader = File::open(workspace.as_ref().join(path))?;
                #[allow(deprecated)]
                archive.start_file_from_path(path, options.clone())?;
                let mut buffer = [0; 10240];
                loop {
                    let count = reader.read(&mut buffer)?;
                    if count == 0 {
                        break;
                    }
                    archive.write(&buffer[..count])?;
                }
            }
            _ => {}
        }
    }
    archive.finish()?;
    Ok(())
}

pub fn unarchive_changes<P>(workspace: P, archive: P) -> Result<()>
where
    P: AsRef<Path>,
{
    println!("* Unarchive changes from: {:?}", archive.as_ref());
    let mut archive = ZipArchive::new(File::open(archive)?)?;
    let changes = serde_json::from_slice::<HashMap<PathBuf, Change>>(archive.comment())?;
    for (path, change) in changes {
        match change {
            Change::Add | Change::Update => match archive.by_name(&path.to_string_lossy()) {
                Ok(mut reader) => {
                    println!("* Unarchive change: {:?}", path);
                    let mut writer = File::create(workspace.as_ref().join(&path))?;
                    let mut buffer = [0; 10240];
                    loop {
                        let count = reader.read(&mut buffer)?;
                        if count == 0 {
                            break;
                        }
                        writer.write(&buffer[..count])?;
                    }
                }
                Err(error) => println!("* Could not update file: {:?} - {:?}", path, error),
            },
            Change::Remove => {
                if let Err(error) = remove_file(workspace.as_ref().join(&path)) {
                    println!("* Could not remove local file: {:?} - {:?}", path, error);
                }
            }
        }
    }
    Ok(())
}
