// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Author: Alexander Seifarth

use super::ast;
use super::parser;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tokio;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub enum ParseError {
    IoError{file: PathBuf, referenced_by: Option<PathBuf>, error: std::io::Error},
    PathError{file: PathBuf, referenced_by: Option<PathBuf>, error: std::io::Error},
    MaxImportNestingReached{file: PathBuf, referenced_by: Option<PathBuf>},
    AlreadyParsed{file: PathBuf, referenced_by: Option<PathBuf>},
    FileNotFound{file: PathBuf,  referenced_by: Option<PathBuf>},
    SyntaxError{file: PathBuf,  referenced_by: Option<PathBuf>,}
}

/// Parses a list of FRANCA IDL files including imported FRANCA files transitively.
/// The method only parses the files and returns them as list of ast::Modules, there is no
/// semantic check.
/// # Arguments
/// * `fidls`:        List of FRANCA IDL (.fidl) files to parse
/// * `search_dirs`:  List of directories used to search for imported FRANCA FIDL files.
/// * `max_import_nesting`: Maximum depth of import file nesting.
pub async fn compile_fidls(fidls: &Vec<PathBuf>, search_dirs: &Vec<PathBuf>, max_import_nesting: usize)
                           ->  (Vec<(ast::Module, PathBuf)>, Vec<ParseError>)  {
    let module_list = Arc::new(Mutex::new(Vec::new()));
    let mut jhs = Vec::new();
    for f in fidls {
        let modules_clone = module_list.clone();
        let jh = tokio::spawn(
            parse_single_fidl(f.clone(), None,modules_clone,
                              search_dirs.clone(), max_import_nesting)
        );
        jhs.push(jh);
    }

    let mut mods = Vec::new();
    let mut errs = Vec::new();
    for jh in jhs {
        let res =  jh.await.unwrap();
        for r in res {
            match r {
                Ok((vmod, path)) => mods.push((vmod, path)),
                Err(err) => errs.push(err),
            }
        }
    }
    (mods, errs)
}

pub async fn parse_single_fidl(filepath: PathBuf,
                               referenced_by: Option<PathBuf>,
                               mod_list: Arc<Mutex<Vec<PathBuf>>>,
                               search_dirs: Vec<PathBuf>,
                               max_nesting: usize)
        -> Vec<Result<(ast::Module, PathBuf), ParseError>>
{
    use std::fs;
    let file = match filepath.canonicalize() {
        Ok(pt) => pt,
        Err(error) => return vec![Err(ParseError::PathError {file: filepath, referenced_by, error})],
    };

    {
        let mut ml_lock = mod_list.lock().unwrap();
        if ml_lock.contains(&file) {
            return vec![]
        }
        ml_lock.push(file.clone());
    }

    let text = match fs::read_to_string(file.clone()) {
        Ok(r) => r,
        Err(error) => return vec![Err(ParseError::IoError {file: file.to_path_buf(), referenced_by, error})]
    };

    let module = match parser::parse_module(&text) {
        Ok(m) => m.1,
        Err(_) => return vec![Err(ParseError::SyntaxError {file: file.to_path_buf(), referenced_by})]
    };

    if module.imports.len() > 0 && max_nesting == 0 {
        return vec![Err(ParseError::MaxImportNestingReached {file: file.to_path_buf(), referenced_by})];
    }

    let mut jhs = Vec::new();
    let current_dir = filepath.parent();
    for imp in module.imports.iter().filter(|i| !i.uri.is_empty()) {
        let referenced_by = file.clone();
        let ml = mod_list.clone();
        let import_file = find_file(&imp.uri, current_dir, &search_dirs);
        if let Some(f) = import_file {
            let jh = spawn(f, Some(referenced_by), ml,
                           search_dirs.clone(), max_nesting -1);
            jhs.push(jh);
        }
        else {
            return vec![Err(ParseError::FileNotFound {file: Path::new(&imp.uri).to_path_buf(), referenced_by: Some(file) })]
        }
    }

    let mut result = vec![Ok((module, file.to_path_buf()))];
    for jh in jhs {
        let mut r = jh.await.unwrap();
        result.append(&mut r);
    }
    result
}

fn spawn(filepath: PathBuf, referenced_by: Option<PathBuf>, mod_list: Arc<Mutex<Vec<PathBuf>>>,
    search_dirs: Vec<PathBuf>, max_nesting: usize)
        -> JoinHandle<Vec<Result<(ast::Module, PathBuf), ParseError>>>
{
    tokio::spawn(async move {
        parse_single_fidl(filepath, referenced_by, mod_list, search_dirs, max_nesting -1).await
    })
}

fn find_file(uri: &str, current_dir: Option<&Path>, search_dirs: &Vec<PathBuf>)
        -> Option<PathBuf> {
    let uri_path = Path::new(uri);
    if uri_path.is_absolute() {
        if uri_path.is_file() {
            return Some(uri_path.to_path_buf())
        }
        return None;
    }

    if let Some(cdir) = current_dir {
        let f1 = cdir.join(uri_path);
        if f1.is_file() {
            return Some(f1);
        }
    }

    for dir in search_dirs {
        let f = dir.join(uri_path);
        if f.is_file() {
            return Some(f);
        }
    }
    None
}
