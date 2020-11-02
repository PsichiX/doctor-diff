use crate::hash::HashValue;
use ring::digest::{Context, SHA256};
use std::{
    collections::HashMap,
    fs::{read_dir, File},
    io::Read,
    io::Result,
    path::{Path, PathBuf},
};

pub fn hash_directory<P>(path: P) -> Result<HashMap<PathBuf, HashValue>>
where
    P: AsRef<Path>,
{
    let mut result = HashMap::new();
    hash_directory_inner(path, &mut result)?;
    Ok(result)
}

fn hash_directory_inner<P>(path: P, result: &mut HashMap<PathBuf, HashValue>) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    if path.is_dir() {
        for entry in read_dir(path)? {
            let entry = entry?;
            let path = path.join(entry.file_name());
            if path.is_dir() {
                hash_directory_inner(path, result)?;
            } else {
                let hash = sha256_stream(File::open(&path)?)?;
                result.insert(path.to_owned(), hash);
            }
        }
    }
    Ok(())
}

pub fn sha256_stream<R>(mut reader: R) -> Result<HashValue>
where
    R: Read,
{
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }
    Ok(HashValue(context.finish().as_ref().to_vec()))
}
