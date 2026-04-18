use crate::ast::ModuleItem;
use crate::parser::Parser as CPParser;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashSet;

pub fn resolve_file_or_dir(base: &Path) -> Option<PathBuf> { 
    // 1. Check <base>.d.cp
    let dcp_file = base.with_extension("d.cp");
    if dcp_file.exists() {
        return Some(dcp_file);
    }
    // 2. Check <base>.cp
    let cp_file = base.with_extension("cp");
    if cp_file.exists() {
        return Some(cp_file);
    }

    if base.is_dir() {
        // 3. Check for pre-compiled types in bin/ (for dependencies)
        if let Some(name) = base.file_name().and_then(|n| n.to_str()) {
            let bin_dcp = base.join("bin").join(format!("{}.d.cp", name));
            if bin_dcp.exists() {
                return Some(bin_dcp);
            }
        }

        // 4. Check directory defaults
        let checks = vec!["index.d.cp", "index.cp", "main.d.cp", "main.cp"];
        for check in checks {
            let p = base.join(check);
            if p.exists() {
                return Some(p);
            }
        }

        // 5. Check cap.json main
        let cap_json = base.join("cap.json");
        if cap_json.exists() {
            if let Ok(content) = fs::read_to_string(&cap_json) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(main_file) = json.get("main").and_then(|v| v.as_str()) {
                        let m_path = PathBuf::from(main_file);
                        let m_base = base.join(m_path.parent().unwrap_or(Path::new(""))).join(m_path.file_stem().unwrap_or(std::ffi::OsStr::new("")));
                        
                        let dcp = m_base.with_extension("d.cp");
                        if dcp.exists() { return Some(dcp); }
                        let cp = m_base.with_extension("cp");
                        if cp.exists() { return Some(cp); }
                    }
                }
            }
        }
    }
    None
}

pub fn resolve_dependencies(
    path: &Path,
    include_paths: &[PathBuf],
    all_items: &mut Vec<ModuleItem>,
    processed: &mut HashSet<PathBuf>,
) -> anyhow::Result<()> {
    let abs_path = fs::canonicalize(path)?;
    processed.insert(abs_path.clone());

    let code = fs::read_to_string(path)?;
    let mut parser = CPParser::new(&code);
    let module = parser.parse_module().map_err(|e| anyhow::anyhow!("Parse Error in {:?}: {}", path, e))?;

    for item in module.body {
        match item {
            ModuleItem::Import(imp) => {
                let mut search_source = imp.source.clone();
                if let Some(stripped) = search_source.strip_prefix("cap:") {
                    search_source = format!("cap/{}", stripped);
                }

                let mut resolved_path = None;

                if search_source.starts_with("./") || search_source.starts_with("../") || search_source.starts_with("/") || search_source.starts_with("cap/") {
                    let base_path = path.parent().unwrap().join(&search_source);
                    resolved_path = resolve_file_or_dir(&base_path);
                    
                    if resolved_path.is_none() {
                        for inc in include_paths {
                            let p = inc.join(&search_source);
                            if let Some(res) = resolve_file_or_dir(&p) {
                                resolved_path = Some(res);
                                break;
                            }
                        }
                    }
                } else {
                    let mut current_dir = path.parent().unwrap();
                    loop {
                        let dep_path = current_dir.join("cap_modules").join(&search_source);
                        if let Some(p) = resolve_file_or_dir(&dep_path) {
                            resolved_path = Some(p);
                            break;
                        }
                        if let Some(parent) = current_dir.parent() {
                            current_dir = parent;
                        } else {
                            break;
                        }
                    }

                    if resolved_path.is_none() {
                        let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_else(|_| String::from("~"));
                        let global_path = PathBuf::from(&home).join(".cap/global/cap_modules").join(&search_source);
                        resolved_path = resolve_file_or_dir(&global_path);
                    }
                }

                if let Some(p) = resolved_path {
                    resolve_imports(&p, include_paths, all_items, processed)?;
                } else {
                    all_items.push(ModuleItem::Import(imp));
                }
            }
            _ => {}
        }
    }

    Ok(())
}

pub fn resolve_imports(
    path: &Path,
    include_paths: &[PathBuf],
    all_items: &mut Vec<ModuleItem>,
    processed: &mut HashSet<PathBuf>,
) -> anyhow::Result<()> {
    
    let abs_path = fs::canonicalize(path)?;
    if processed.contains(&abs_path) {
        return Ok(());
    }
    processed.insert(abs_path.clone());

    let code = fs::read_to_string(path)?;
    let mut parser = CPParser::new(&code);
    let module = parser.parse_module().map_err(|e| anyhow::anyhow!("Parse Error in {:?}: {}", path, e))?;

    for item in module.body {
        match item {
            ModuleItem::Import(imp) => {
                let mut search_source = imp.source.clone();
                if let Some(stripped) = search_source.strip_prefix("cap:") {
                    search_source = format!("cap/{}", stripped);
                }

                let mut resolved_path = None;

                if search_source.starts_with("./") || search_source.starts_with("../") || search_source.starts_with("/") || search_source.starts_with("cap/") {
                    // Relative, absolute or cap: import
                    let base_path = path.parent().unwrap().join(&search_source);
                    resolved_path = resolve_file_or_dir(&base_path);
                    
                    // Fallback for include paths if still not found (legacy behavior)
                    if resolved_path.is_none() {
                        for inc in include_paths {
                            let p = inc.join(&search_source);
                            if let Some(res) = resolve_file_or_dir(&p) {
                                resolved_path = Some(res);
                                break;
                            }
                        }
                    }
                } else {
                    // Dependency import (search upwards for cap_modules)
                    let mut current_dir = path.parent().unwrap();
                    loop {
                        let dep_path = current_dir.join("cap_modules").join(&search_source);
                        if let Some(p) = resolve_file_or_dir(&dep_path) {
                            resolved_path = Some(p);
                            break;
                        }
                        if let Some(parent) = current_dir.parent() {
                            current_dir = parent;
                        } else {
                            break;
                        }
                    }

                    // Fallback to global installation
                    if resolved_path.is_none() {
                        let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_else(|_| String::from("~"));
                        let global_path = PathBuf::from(&home).join(".cap/global/cap_modules").join(&search_source);
                        resolved_path = resolve_file_or_dir(&global_path);
                    }
                }

                if let Some(p) = resolved_path {
                    resolve_imports(&p, include_paths, all_items, processed)?;
                } else {
                    all_items.push(ModuleItem::Import(imp));
                }
            }
            _ => {
                all_items.push(item);
            }
        }
    }

    Ok(())
}
