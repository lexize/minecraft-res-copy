use std::env::args;

use regex::Regex;

pub struct Parameters {
    pub index_file: String,
    pub objects_folder: String,
    pub output_folder: String,
    pub match_pattern: Regex,
    pub symlink: bool,
    pub verbose: bool
}

pub fn read_from_args() -> Result<Parameters, ()> {
    let mut index = None;
    let mut objects = None;
    let mut output = None;
    let mut pattern = None;
    let mut symlink = false;
    let mut verbose = false;
    let mut args = args();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-i" | "--index" => {
                index = args.next();
            }
            "-o" | "--objects" => {
                objects = args.next();
            }
            "-O" | "--output" => {
                output = args.next();
            }
            "-P" | "--pattern" => {
                pattern = args.next();
            }
            "-s" | "--symlink" => {
                symlink = true;
            }
            "-v" | "--verbose" => {
                verbose = true;
            }
            _ => {}
        }
    }
    let mut error_stack = Vec::new();
    if index.is_none() {
        error_stack.push("Index is required to copy assets. Set with \"--index [path/to/index.json]\"".to_string())
    }
    if objects.is_none() {
        error_stack.push("Objects folder is required to copy assets. Set with \"--index [path/to/index.json]\"".to_string())
    }
    let pat = pattern.get_or_insert(".*".to_string());
    let regex = regex::Regex::new(pat.as_str());
    if let Err(e) = &regex {
        error_stack.push(format!("Error occured while parsing pattern: {}", e.to_string()))
    }

    if !error_stack.is_empty() {
        crate::error(error_stack.join("\n"))?
    }

    Ok(Parameters { 
        index_file: index.unwrap(), 
        objects_folder: objects.unwrap(), 
        output_folder: output.unwrap_or("out".to_string()), 
        match_pattern: regex.unwrap(), 
        symlink,
        verbose
    })
}