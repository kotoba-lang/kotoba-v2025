//! JSON to JSON-LD Converter
//!
//! This tool converts JSON Schema files and JSON data files to JSON-LD format
//! by adding @context references and mapping properties to semantic vocabulary.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde_json::{json, Value, Map};
use anyhow::{Context, Result};
use clap::{Parser, ArgGroup};

#[derive(Parser)]
#[command(name = "json-to-jsonld")]
#[command(about = "Convert JSON Schema and JSON files to JSON-LD format")]
#[command(version = "0.1.0")]
struct Cli {
    /// Input JSON file or directory
    #[arg(short, long)]
    input: PathBuf,
    
    /// Output JSON-LD file or directory
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// Context file path (default: schemas/kotoba-context.jsonld)
    #[arg(short, long, default_value = "schemas/kotoba-context.jsonld")]
    context: PathBuf,
    
    /// Include context inline (default: use @context reference)
    #[arg(long)]
    inline_context: bool,
    
    /// Process directory recursively
    #[arg(short, long)]
    recursive: bool,
    
    /// Keep original JSON files (don't delete after conversion)
    #[arg(long)]
    keep_original: bool,
    
    /// Validate JSON-LD output
    #[arg(long)]
    validate: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Load context file
    let context_value = if cli.context.exists() {
        let context_str = fs::read_to_string(&cli.context)
            .with_context(|| format!("Failed to read context file: {}", cli.context.display()))?;
        serde_json::from_str::<Value>(&context_str)
            .with_context(|| "Failed to parse context file as JSON")?
    } else {
        eprintln!("Warning: Context file not found: {}", cli.context.display());
        json!({})
    };
    
    // Determine output path
    let output_path = cli.output.clone().unwrap_or_else(|| {
        if cli.input.is_file() {
            let mut path = cli.input.clone();
            path.set_extension("jsonld");
            path
        } else {
            cli.input.clone()
        }
    });
    
    // Process input
    if cli.input.is_file() {
        convert_file(&cli.input, &output_path, &context_value, cli.inline_context, cli.validate)?;
        if !cli.keep_original && cli.input != output_path {
            fs::remove_file(&cli.input)
                .with_context(|| format!("Failed to remove original file: {}", cli.input.display()))?;
        }
    } else if cli.input.is_dir() {
        convert_directory(&cli.input, &output_path, &context_value, cli.inline_context, cli.recursive, cli.keep_original, cli.validate)?;
    } else {
        anyhow::bail!("Input path does not exist: {}", cli.input.display());
    }
    
    println!("Conversion completed successfully!");
    Ok(())
}

fn convert_directory(
    input_dir: &Path,
    output_dir: &Path,
    context: &Value,
    inline_context: bool,
    recursive: bool,
    keep_original: bool,
    validate: bool,
) -> Result<()> {
    // Create output directory if it doesn't exist
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)
            .with_context(|| format!("Failed to create output directory: {}", output_dir.display()))?;
    }
    
    let entries = fs::read_dir(input_dir)
        .with_context(|| format!("Failed to read directory: {}", input_dir.display()))?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
            // Skip if it's already a JSON-LD file or package.json/tsconfig.json
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if file_name == "package.json" || file_name == "tsconfig.json" || file_name.ends_with(".jsonld") {
                continue;
            }
            
            let output_file = output_dir.join(path.file_name().unwrap()).with_extension("jsonld");
            convert_file(&path, &output_file, context, inline_context, validate)?;
            
            if !keep_original {
                fs::remove_file(&path)
                    .with_context(|| format!("Failed to remove original file: {}", path.display()))?;
            }
        } else if path.is_dir() && recursive {
            let sub_output = output_dir.join(path.file_name().unwrap());
            convert_directory(&path, &sub_output, context, inline_context, recursive, keep_original, validate)?;
        }
    }
    
    Ok(())
}

fn convert_file(
    input_path: &Path,
    output_path: &Path,
    context: &Value,
    inline_context: bool,
    validate: bool,
) -> Result<()> {
    println!("Converting: {} -> {}", input_path.display(), output_path.display());
    
    // Read input JSON
    let json_str = fs::read_to_string(input_path)
        .with_context(|| format!("Failed to read input file: {}", input_path.display()))?;
    
    let mut json_value: Value = serde_json::from_str(&json_str)
        .with_context(|| format!("Failed to parse JSON: {}", input_path.display()))?;
    
    // Convert to JSON-LD
    let jsonld_value = convert_to_jsonld(&mut json_value, context, inline_context, input_path)?;
    
    // Write output
    let output_str = serde_json::to_string_pretty(&jsonld_value)
        .with_context(|| "Failed to serialize JSON-LD")?;
    
    // Create parent directory if needed
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory: {}", parent.display()))?;
    }
    
    fs::write(output_path, output_str)
        .with_context(|| format!("Failed to write output file: {}", output_path.display()))?;
    
    // Validate if requested
    if validate {
        validate_jsonld(&jsonld_value)?;
    }
    
    Ok(())
}

fn convert_to_jsonld(
    json_value: &mut Value,
    context: &Value,
    inline_context: bool,
    file_path: &Path,
) -> Result<Value> {
    let mut result = Map::new();
    
    // Add @context
    if inline_context {
        result.insert("@context".to_string(), context.clone());
    } else {
        // Use reference to context file
        let context_ref = if file_path.starts_with("schemas/") {
            "kotoba-context.jsonld".to_string()
        } else {
            format!("../schemas/kotoba-context.jsonld")
        };
        result.insert("@context".to_string(), json!(context_ref));
    }
    
    // Convert JSON Schema to JSON-LD
    if is_json_schema(json_value) {
        convert_json_schema_to_jsonld(json_value, &mut result)?;
    } else {
        // Regular JSON data - add @type based on file name or content
        add_type_from_filename(file_path, &mut result);
        result.insert("@id".to_string(), json!(format!("file://{}", file_path.display())));
        
        // Merge original content
        if let Value::Object(obj) = json_value {
            for (key, value) in obj {
                if !result.contains_key(&key) {
                    result.insert(key, value);
                }
            }
        } else {
            result.insert("@value".to_string(), json_value.clone());
        }
    }
    
    Ok(Value::Object(result))
}

fn is_json_schema(value: &Value) -> bool {
    if let Value::Object(obj) = value {
        obj.contains_key("$schema") || obj.contains_key("$id") || obj.contains_key("properties") || obj.contains_key("$defs")
    } else {
        false
    }
}

fn convert_json_schema_to_jsonld(schema: &Value, result: &mut Map<String, Value>) -> Result<()> {
    if let Value::Object(obj) = schema {
        // Add @id from $id
        if let Some(id) = obj.get("$id") {
            result.insert("@id".to_string(), id.clone());
        }
        
        // Add @type
        if let Some(title) = obj.get("title") {
            if let Some(title_str) = title.as_str() {
                result.insert("@type".to_string(), json!(format!("kotoba:{}", title_str.replace(" ", ""))));
            }
        }
        
        // Convert title to rdfs:label
        if let Some(title) = obj.get("title") {
            result.insert("rdfs:label".to_string(), title.clone());
        }
        
        // Convert description to rdfs:comment
        if let Some(desc) = obj.get("description") {
            result.insert("rdfs:comment".to_string(), desc.clone());
        }
        
        // Convert properties to shacl:property
        if let Some(properties) = obj.get("properties") {
            if let Value::Object(props) = properties {
                let mut shacl_props = Vec::new();
                for (prop_name, prop_schema) in props {
                    let mut prop_obj = Map::new();
                    prop_obj.insert("shacl:path".to_string(), json!(format!("kotoba:{}", prop_name)));
                    
                    if let Value::Object(prop_obj_inner) = prop_schema {
                        // Map JSON Schema types to XSD types
                        if let Some(prop_type) = prop_obj_inner.get("type") {
                            let xsd_type = match prop_type.as_str() {
                                Some("string") => "xsd:string",
                                Some("integer") => "xsd:integer",
                                Some("number") => "xsd:double",
                                Some("boolean") => "xsd:boolean",
                                Some("array") => "rdf:List",
                                Some("object") => "rdf:JSON",
                                _ => "xsd:string",
                            };
                            prop_obj.insert("shacl:datatype".to_string(), json!(xsd_type));
                        }
                        
                        // Add description
                        if let Some(desc) = prop_obj_inner.get("description") {
                            prop_obj.insert("rdfs:comment".to_string(), desc.clone());
                        }
                        
                        // Handle format
                        if let Some(format) = prop_obj_inner.get("format") {
                            if format.as_str() == Some("date-time") {
                                prop_obj.insert("shacl:datatype".to_string(), json!("xsd:dateTime"));
                            } else if format.as_str() == Some("email") {
                                prop_obj.insert("shacl:datatype".to_string(), json!("xsd:string"));
                            }
                        }
                    }
                    
                    shacl_props.push(Value::Object(prop_obj));
                }
                result.insert("shacl:property".to_string(), json!(shacl_props));
            }
        }
        
        // Convert required fields
        if let Some(required) = obj.get("required") {
            if let Value::Array(req_array) = required {
                let mut req_props = Vec::new();
                for req_field in req_array {
                    if let Some(req_str) = req_field.as_str() {
                        req_props.push(json!({
                            "shacl:path": format!("kotoba:{}", req_str),
                            "shacl:minCount": 1
                        }));
                    }
                }
                if !req_props.is_empty() {
                    // Merge with existing shacl:property if any
                    if let Some(existing_props) = result.get_mut("shacl:property") {
                        if let Value::Array(existing_array) = existing_props {
                            // Update existing properties to mark as required
                            for prop in existing_array.iter_mut() {
                                if let Value::Object(prop_obj) = prop {
                                    if let Some(path) = prop_obj.get("shacl:path") {
                                        if let Some(path_str) = path.as_str() {
                                            for req_prop in &req_props {
                                                if let Value::Object(req_obj) = req_prop {
                                                    if let Some(req_path) = req_obj.get("shacl:path") {
                                                        if req_path == path {
                                                            prop_obj.insert("shacl:minCount".to_string(), json!(1));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        result.insert("shacl:property".to_string(), json!(req_props));
                    }
                }
            }
        }
        
        // Handle $defs
        if let Some(defs) = obj.get("$defs") {
            result.insert("kotoba:definitions".to_string(), defs.clone());
        }
        
        // Preserve custom kotoba:* properties
        for (key, value) in obj {
            if key.starts_with("kotoba:") {
                result.insert(key.clone(), value.clone());
            }
        }
        
        // Preserve other schema metadata
        if let Some(meta) = obj.get("meta") {
            result.insert("kotoba:meta".to_string(), meta.clone());
        }
    }
    
    Ok(())
}

fn add_type_from_filename(file_path: &Path, result: &mut Map<String, Value>) {
    if let Some(file_stem) = file_path.file_stem().and_then(|s| s.to_str()) {
        // Map common file names to types
        let type_name = match file_stem {
            "topology_data" => "kotoba:TopologyGraph",
            "dag" => "kotoba:ProcessNetwork",
            "user" => "kotoba:User",
            "follows" => "kotoba:Follows",
            _ => "kotoba:Document",
        };
        result.insert("@type".to_string(), json!(type_name));
    }
}

fn validate_jsonld(jsonld: &Value) -> Result<()> {
    // Basic validation: check for @context
    if let Value::Object(obj) = jsonld {
        if !obj.contains_key("@context") {
            anyhow::bail!("JSON-LD document missing @context");
        }
        
        // Check for valid @context
        if let Some(context) = obj.get("@context") {
            match context {
                Value::String(_) => {
                    // Reference to external context - OK
                }
                Value::Object(_) => {
                    // Inline context - OK
                }
                _ => {
                    anyhow::bail!("Invalid @context format");
                }
            }
        }
    } else {
        anyhow::bail!("JSON-LD document must be an object");
    }
    
    println!("Validation passed");
    Ok(())
}

