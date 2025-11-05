//! Extract metadata from Cargo.toml files and generate JSON-LD metadata files
//! for each crate in the workspace.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize)]
struct CargoToml {
    package: Option<Package>,
    dependencies: Option<toml::Value>,
    #[serde(rename = "dev-dependencies")]
    dev_dependencies: Option<toml::Value>,
    features: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: String,
    version: Option<String>,
    edition: Option<String>,
    license: Option<String>,
    repository: Option<String>,
    description: Option<String>,
    keywords: Option<Vec<String>>,
    categories: Option<Vec<String>>,
}

fn find_crate_layer(crate_path: &Path) -> Option<String> {
    for component in crate_path.components() {
        let name = component.as_os_str().to_string_lossy();
        if name.starts_with("010-") || name.starts_with("020-") || name.starts_with("030-") ||
           name.starts_with("040-") || name.starts_with("050-") || name.starts_with("060-") ||
           name.starts_with("070-") || name.starts_with("090-") || name.starts_with("005-") {
            return Some(name.to_string());
        }
    }
    None
}

fn relative_context_path(crate_path: &Path, workspace_root: &Path) -> String {
    let depth = crate_path.strip_prefix(workspace_root)
        .map(|p| p.components().count())
        .unwrap_or(0);
    
    if depth == 0 {
        "schemas/kotoba-context.jsonld".to_string()
    } else {
        format!("{}{}", "../".repeat(depth), "schemas/kotoba-context.jsonld")
    }
}

fn parse_dependency(dep_name: &str, dep_value: &toml::Value) -> serde_json::Value {
    match dep_value {
        toml::Value::String(version) => {
            json!({
                "name": dep_name,
                "type": "external",
                "version": version
            })
        }
        toml::Value::Table(table) => {
            let mut result = json!({
                "name": dep_name
            });
            
            if let Some(path) = table.get("path").and_then(|v| v.as_str()) {
                result["type"] = json!("workspace");
                result["path"] = json!(path);
            } else {
                result["type"] = json!("external");
            }
            
            if let Some(version) = table.get("version").and_then(|v| v.as_str()) {
                result["version"] = json!(version);
            }
            
            if let Some(optional) = table.get("optional").and_then(|v| v.as_bool()) {
                if optional {
                    result["optional"] = json!(true);
                }
            }
            
            if let Some(features) = table.get("features").and_then(|v| v.as_array()) {
                let feat_vec: Vec<String> = features.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                result["features"] = json!(feat_vec);
            }
            
            if let Some(workspace) = table.get("workspace").and_then(|v| v.as_bool()) {
                if workspace {
                    result["workspace"] = json!(true);
                }
            }
            
            result
        }
        _ => json!({
            "name": dep_name,
            "type": "unknown"
        })
    }
}

fn extract_crate_metadata(crate_path: &Path, workspace_root: &Path) -> Option<serde_json::Value> {
    let cargo_toml = crate_path.join("Cargo.toml");
    
    if !cargo_toml.exists() {
        return None;
    }
    
    let content = match fs::read_to_string(&cargo_toml) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading {}: {}", cargo_toml.display(), e);
            return None;
        }
    };
    
    let cargo_data: CargoToml = match toml::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error parsing {}: {}", cargo_toml.display(), e);
            return None;
        }
    };
    
    let package = match cargo_data.package {
        Some(p) => p,
        None => return None,
    };
    
    let relative_path = crate_path.strip_prefix(workspace_root)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| crate_path.to_string_lossy().to_string());
    
    let layer = find_crate_layer(crate_path);
    
    // Parse dependencies
    let mut workspace_deps = Vec::new();
    let mut external_deps = Vec::new();
    let mut optional_deps = Vec::new();
    
    if let Some(deps) = cargo_data.dependencies {
        if let toml::Value::Table(table) = deps {
            for (dep_name, dep_value) in table {
                let dep_info = parse_dependency(&dep_name, &dep_value);
                
                if dep_info.get("type").and_then(|v| v.as_str()) == Some("workspace") {
                    workspace_deps.push(dep_info);
                } else if dep_info.get("optional").and_then(|v| v.as_bool()) == Some(true) {
                    optional_deps.push(dep_info);
                } else {
                    external_deps.push(dep_info);
                }
            }
        }
    }
    
    let mut dev_deps = Vec::new();
    if let Some(deps) = cargo_data.dev_dependencies {
        if let toml::Value::Table(table) = deps {
            for (dep_name, dep_value) in table {
                dev_deps.push(parse_dependency(&dep_name, &dep_value));
            }
        }
    }
    
    // Extract features
    let default_features = cargo_data.features
        .as_ref()
        .and_then(|f| f.get("default").cloned())
        .unwrap_or_default();
    
    let optional_features: Vec<String> = cargo_data.features
        .as_ref()
        .map(|f| f.keys().filter(|k| *k != "default").cloned().collect())
        .unwrap_or_default();
    
    // Find source files
    let src_dir = crate_path.join("src");
    let mut source_files = Vec::new();
    if src_dir.exists() {
        if let Ok(entries) = fs::read_dir(&src_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    if let Ok(rel_path) = path.strip_prefix(crate_path) {
                        source_files.push(rel_path.to_string_lossy().to_string());
                    }
                } else if path.is_dir() {
                    // Recursively find .rs files
                    if let Ok(walker) = walkdir::WalkDir::new(&path).into_iter().collect::<Result<Vec<_>, _>>() {
                        for entry in walker {
                            if entry.file_type().is_file() {
                                if let Some(ext) = entry.path().extension() {
                                    if ext == "rs" {
                                        if let Ok(rel_path) = entry.path().strip_prefix(crate_path) {
                                            source_files.push(rel_path.to_string_lossy().to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    source_files.sort();
    
    // Build metadata JSON-LD
    let mut metadata = json!({
        "@context": relative_context_path(crate_path, workspace_root),
        "@type": "kotoba:Crate",
        "@id": format!("kotoba:crate/{}", package.name),
        "name": package.name,
        "version": package.version.unwrap_or_default(),
        "path": relative_path,
        "status": "active",
    });
    
    if let Some(edition) = package.edition {
        metadata["edition"] = json!(edition);
    }
    if let Some(license) = package.license {
        metadata["license"] = json!(license);
    }
    if let Some(repository) = package.repository {
        metadata["repository"] = json!(repository);
    }
    if let Some(description) = package.description {
        metadata["description"] = json!(description);
    }
    if let Some(keywords) = package.keywords {
        metadata["keywords"] = json!(keywords);
    }
    if let Some(categories) = package.categories {
        metadata["categories"] = json!(categories);
    }
    
    if let Some(layer_name) = layer {
        metadata["layer"] = json!(format!("kotoba:layer/{}", layer_name));
    }
    
    metadata["dependencies"] = json!({
        "workspace": workspace_deps,
        "external": external_deps,
        "optional": optional_deps,
        "dev": dev_deps,
    });
    
    metadata["features"] = json!({
        "default": default_features,
        "optional": optional_features,
    });
    
    if !source_files.is_empty() {
        metadata["sourceFiles"] = json!(source_files);
    }
    
    Some(metadata)
}

fn main() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let crates_dir = workspace_root.join("crates");
    
    if !crates_dir.exists() {
        eprintln!("Error: {} does not exist", crates_dir.display());
        std::process::exit(1);
    }
    
    // Find all Cargo.toml files
    let mut cargo_files = Vec::new();
    if let Ok(entries) = fs::read_dir(&crates_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                find_cargo_toml_files(&path, &mut cargo_files);
            }
        }
    }
    
    println!("Found {} Cargo.toml files", cargo_files.len());
    
    // Extract metadata for each crate
    for cargo_file in cargo_files {
        let crate_path = cargo_file.parent().unwrap();
        if let Some(metadata) = extract_crate_metadata(crate_path, &workspace_root) {
            let metadata_file = crate_path.join("metadata.jsonld");
            if let Ok(json_str) = serde_json::to_string_pretty(&metadata) {
                if let Err(e) = fs::write(&metadata_file, json_str) {
                    eprintln!("Error writing {}: {}", metadata_file.display(), e);
                } else {
                    println!("Generated: {}", metadata_file.display());
                }
            }
        } else {
            // Mark as unimplemented
            let relative_path = crate_path.strip_prefix(&workspace_root)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| crate_path.to_string_lossy().to_string());
            let layer = find_crate_layer(crate_path);
            
            let mut unimplemented_metadata = json!({
                "@context": relative_context_path(crate_path, &workspace_root),
                "@type": "kotoba:Crate",
                "@id": format!("kotoba:crate/{}", crate_path.file_name().unwrap().to_string_lossy()),
                "name": crate_path.file_name().unwrap().to_string_lossy(),
                "path": relative_path,
                "status": "unimplemented",
            });
            
            if let Some(layer_name) = layer {
                unimplemented_metadata["layer"] = json!(format!("kotoba:layer/{}", layer_name));
            }
            
            let metadata_file = crate_path.join("metadata.jsonld");
            if let Ok(json_str) = serde_json::to_string_pretty(&unimplemented_metadata) {
                if let Err(e) = fs::write(&metadata_file, json_str) {
                    eprintln!("Error writing {}: {}", metadata_file.display(), e);
                } else {
                    println!("Marked as unimplemented: {}", metadata_file.display());
                }
            }
        }
    }
}

fn find_cargo_toml_files(dir: &Path, results: &mut Vec<PathBuf>) {
    let cargo_toml = dir.join("Cargo.toml");
    if cargo_toml.exists() {
        results.push(cargo_toml);
    }
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                find_cargo_toml_files(&path, results);
            }
        }
    }
}

