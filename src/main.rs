use std::{fs::{self, File}, io::{BufReader, BufWriter, Read, Write}, path::{Path, PathBuf}};

use json::JsonValue;

mod params;

fn main() -> Result<(), ()> {
    let param = params::read_from_args()?;
    let out_path = Path::new(&param.output_folder).to_path_buf();
    // Creating an output directory
    mkdir_all(&out_path)?;
    let json_contents = read_string(&param.index_file)?;
    let jobject;
    if let JsonValue::Object(o) = json::parse(json_contents.as_str()).or_else(|_| error("Unable to parse JSON"))? {
        jobject = o;
    }
    else {
        error("Invalid JSON contents")?;
        unreachable!()
    }
    // Reading index file
    if let Some(objects) = jobject.get("objects") {
        if let JsonValue::Object(o) = objects {
            let objects_path = Path::new(&param.objects_folder);
            for (path, v) in o.iter() {
                if param.match_pattern.captures(path).is_some() {
                    let output_file = out_path.join(path);
                    let hash = if let JsonValue::Object(ob) = v {
                        ob.get("hash").unwrap().as_str().unwrap()
                    }
                    else {
                        let _ = error::<(),_>(format!("Unable to read hash for: {}", path));
                        continue;
                    };
                    let hash_path = if let Ok(h) = get_hash_path(objects_path, hash) {
                        h
                    } else { continue; };
                    if let Err(()) = mkdir_all(output_file.parent().unwrap()) {
                        let _ = error::<(), _>(format!("Unable to create parent folders for {}", output_file.to_str().unwrap()));
                        continue;
                    }
                    if param.symlink {
                        if let Err(e) = symlink(hash_path, &output_file) {
                            let _ = error::<(), _>(format!("Error occured while creating symlink: {}", e.to_string()));
                            continue;
                        }
                    }
                    else {
                        if let Err(_) = copy_to(hash_path, &output_file) {
                            continue;
                        }
                    }
                    if param.verbose {
                        println!("{} done", path);
                    }
                }
            }
        }
        else {
            error("\"objects\" field must be an object")?;
        }
    }
    else {
        error("No \"objects\" field")?;
    }

    Ok(())
}

pub fn error<A, S: ToString>(err: S) -> Result<A, ()> {
    println!("{}", err.to_string());
    Err(())
}

pub fn mkdir_all<P: AsRef<Path>>(path: P) -> Result<(), ()> {
    let path = path.as_ref();
    if let Err(e) = fs::create_dir_all(path) {
        match e.kind() {
            std::io::ErrorKind::AlreadyExists => {
                if !path.is_dir() {
                    error(format!("Path \"{}\" is not a directory", path.to_str().unwrap()))?
                }
            },
            _ => {
                error(format!("Unable to create directory \"{}\":{}", path.to_str().unwrap(), e.to_string()))?
            }
        }
    }
    Ok(())
}

pub fn read_string<P: AsRef<Path>>(path: P) -> Result<String, ()> {
    let path = path.as_ref();
    match fs::read_to_string(path) {
        Ok(s) => Ok(s),
        Err(e) => {
            error(format!("Unable to read file \"{}\":{}", path.to_str().unwrap(), e.to_string()))?
        },
    }
}

pub fn copy_to<P1: AsRef<Path>, P2: AsRef<Path>>(from: P1, to: P2) -> Result<(), ()> {
    let p1 = from.as_ref();
    let p2 = to.as_ref();
    let input = File::open(p1).ok()
    .ok_or_else(|| error::<(),_>(format!("Unable to open input file for {}", p1.to_str().unwrap())).unwrap_err())?;
    let output = File::create(p2).ok()
    .ok_or_else(|| error::<(),_>(format!("Unable to open output file for {}", p2.to_str().unwrap())).unwrap_err())?;
    let mut reader = BufReader::new(input);
    let mut writer = BufWriter::new(output);
    let mut buf = [0u8; 1024];
    while let Ok(r) = reader.read(&mut buf) {
        if r == 0 { break }
        if let Err(e) = writer.write(&buf[0..r]) {
            error(format!("Error writing to {}: {}", p2.to_str().unwrap(), e.to_string()))?;
        }
    }
    Ok(())
}

pub fn get_hash_path<P: AsRef<Path>>(objects_folder: P, hash: &str) -> Result<PathBuf, ()> {
    let p = objects_folder.as_ref().join(&hash[0..2]).join(hash);
    if p.exists() {
        Ok(p)
    }
    else {
        error(format!("Path \"{}\" doesn't exist", p.to_str().unwrap()))
    }
}

#[cfg(target_family="unix")]
pub fn symlink<P: AsRef<Path>, P2: AsRef<Path>>(original_path: P, destination: P2) -> Result<(), std::io::Error> {
    use std::os::unix::fs::symlink;

    symlink(original_path, destination)
}