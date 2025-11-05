#!/usr/bin/env python3
"""
Extract metadata from Cargo.toml files and generate JSON-LD metadata files
for each crate in the workspace.
"""

import os
import json
import re
from pathlib import Path
from typing import Dict, List, Optional, Any
from collections import defaultdict

def parse_toml_value(value_str: str) -> Any:
    """Simple TOML value parser."""
    value_str = value_str.strip()
    
    # String
    if value_str.startswith('"') and value_str.endswith('"'):
        return value_str[1:-1]
    if value_str.startswith("'") and value_str.endswith("'"):
        return value_str[1:-1]
    
    # Boolean
    if value_str.lower() == "true":
        return True
    if value_str.lower() == "false":
        return False
    
    # Inline table (dictionary)
    if value_str.startswith("{") and value_str.endswith("}"):
        table_content = value_str[1:-1].strip()
        if not table_content:
            return {}
        
        result = {}
        # Simple key-value parsing for inline tables
        # This handles: { path = "../x", optional = true }
        current_key = None
        current_value = None
        in_string = False
        quote_char = None
        brace_depth = 0
        bracket_depth = 0
        buffer = ""
        
        i = 0
        while i < len(table_content):
            char = table_content[i]
            
            if char in ('"', "'") and (i == 0 or table_content[i-1] != '\\'):
                if not in_string:
                    in_string = True
                    quote_char = char
                elif char == quote_char:
                    in_string = False
                    quote_char = None
                buffer += char
            elif char == '{' and not in_string:
                brace_depth += 1
                buffer += char
            elif char == '}' and not in_string:
                brace_depth -= 1
                buffer += char
            elif char == '[' and not in_string:
                bracket_depth += 1
                buffer += char
            elif char == ']' and not in_string:
                bracket_depth -= 1
                buffer += char
            elif char == '=' and not in_string and brace_depth == 0 and bracket_depth == 0:
                if current_key is None:
                    current_key = buffer.strip()
                    buffer = ""
            elif char == ',' and not in_string and brace_depth == 0 and bracket_depth == 0:
                if current_key is not None:
                    current_value = buffer.strip()
                    result[current_key] = parse_toml_value(current_value)
                    current_key = None
                    current_value = None
                    buffer = ""
                else:
                    buffer += char
            else:
                buffer += char
            
            i += 1
        
        # Handle last key-value pair
        if current_key is not None:
            current_value = buffer.strip()
            result[current_key] = parse_toml_value(current_value)
        
        return result
    
    # Array
    if value_str.startswith("[") and value_str.endswith("]"):
        items = []
        content = value_str[1:-1].strip()
        if content:
            # Simple array parsing
            for item in re.split(r',(?![^\[\]]*\])', content):
                item = item.strip()
                if item:
                    items.append(parse_toml_value(item))
        return items
    
    # Number
    try:
        if '.' in value_str:
            return float(value_str)
        return int(value_str)
    except ValueError:
        pass
    
    return value_str

def parse_simple_toml(content: str) -> Dict[str, Any]:
    """Simple TOML parser for Cargo.toml files."""
    result = {}
    current_section = None
    current_table = {}
    in_multiline_string = False
    multiline_buffer = []
    current_key = None
    
    for line in content.split('\n'):
        original_line = line
        line = line.rstrip()
        
        # Skip comments and empty lines (unless in multiline)
        if not in_multiline_string:
            # Remove inline comments
            if '#' in line:
                # Check if # is inside a string
                in_string = False
                quote_char = None
                for i, char in enumerate(line):
                    if char in ('"', "'") and (i == 0 or line[i-1] != '\\'):
                        if not in_string:
                            in_string = True
                            quote_char = char
                        elif char == quote_char:
                            in_string = False
                            quote_char = None
                    elif char == '#' and not in_string:
                        line = line[:i]
                        break
            
            line = line.strip()
            if not line:
                continue
        
        # Handle multiline strings
        if in_multiline_string:
            multiline_buffer.append(original_line)
            if '"""' in line or "'''" in line:
                in_multiline_string = False
                # Join and parse multiline value
                multiline_value = '\n'.join(multiline_buffer)
                if current_key:
                    current_table[current_key] = multiline_value.strip('"\'')
                multiline_buffer = []
                current_key = None
            continue
        
        # Section header
        if line.startswith('[') and line.endswith(']'):
            if current_section:
                result[current_section] = current_table
            current_section = line[1:-1]
            current_table = {}
            continue
        
        # Key-value pair
        if '=' in line:
            key, value = line.split('=', 1)
            key = key.strip()
            value_str = value.strip()
            
            # Check for multiline string start
            if value_str.startswith('"""') or value_str.startswith("'''"):
                in_multiline_string = True
                multiline_buffer = [original_line]
                current_key = key
                continue
            
            # Check for inline table
            if value_str.startswith('{') and not value_str.endswith('}'):
                # Multi-line inline table
                table_lines = [value_str]
                brace_count = value_str.count('{') - value_str.count('}')
                # Continue reading until braces balance
                # This is simplified - full implementation would need more parsing
                pass
            
            value = parse_toml_value(value_str)
            current_table[key] = value
    
    if current_section:
        result[current_section] = current_table
    
    return result

KOTOBA_CONTEXT = "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld"

def find_crate_layer(crate_path: str) -> Optional[str]:
    """Determine the layer from crate path."""
    parts = crate_path.split(os.sep)
    for part in parts:
        if part.startswith("010-") or part.startswith("020-") or part.startswith("030-") or \
           part.startswith("040-") or part.startswith("050-") or part.startswith("060-") or \
           part.startswith("070-") or part.startswith("090-") or part.startswith("005-"):
            return part
    return None

def relative_context_path(crate_path: str) -> str:
    """Calculate relative path to schemas/kotoba-context.jsonld."""
    depth = crate_path.count(os.sep)
    if depth == 0:
        return "schemas/kotoba-context.jsonld"
    return "../" * depth + "schemas/kotoba-context.jsonld"

def parse_dependency(dep: Any) -> Dict[str, Any]:
    """Parse a dependency entry from Cargo.toml."""
    if isinstance(dep, str):
        return {"name": dep, "type": "external", "version": dep}
    
    if isinstance(dep, dict):
        result = {}
        
        # Check for path-based dependency (workspace)
        if "path" in dep:
            result["type"] = "workspace"
            result["path"] = dep["path"]
        # Check for workspace = true (workspace dependency)
        elif dep.get("workspace") == True:
            result["type"] = "workspace"
            result["workspace"] = True
        else:
            result["type"] = "external"
        
        if "version" in dep:
            result["version"] = dep["version"]
        
        if "optional" in dep and dep["optional"]:
            result["optional"] = True
        
        if "features" in dep:
            result["features"] = dep["features"]
        
        return result
    
    return {"type": "unknown"}

def extract_crate_metadata(crate_path: Path, workspace_root: Path) -> Optional[Dict[str, Any]]:
    """Extract metadata from a Cargo.toml file."""
    cargo_toml = crate_path / "Cargo.toml"
    
    if not cargo_toml.exists():
        return None
    
    try:
        with open(cargo_toml, 'r', encoding='utf-8') as f:
            content = f.read()
        cargo_data = parse_simple_toml(content)
    except Exception as e:
        print(f"Error parsing {cargo_toml}: {e}")
        return None
    
    package = cargo_data.get("package", {})
    if not package:
        return None
    
    # Extract basic info
    crate_name = package.get("name", "")
    if not crate_name:
        return None
    
    version = package.get("version", "")
    edition = package.get("edition", "")
    license = package.get("license", "")
    repository = package.get("repository", "")
    description = package.get("description", "")
    keywords = package.get("keywords", [])
    if not isinstance(keywords, list):
        keywords = []
    categories = package.get("categories", [])
    if not isinstance(categories, list):
        categories = []
    
    # Determine layer and status
    relative_path = str(crate_path.relative_to(workspace_root))
    layer = find_crate_layer(relative_path)
    
    # Extract dependencies
    dependencies = cargo_data.get("dependencies", {})
    if not isinstance(dependencies, dict):
        dependencies = {}
    
    workspace_deps = []
    external_deps = []
    optional_deps = []
    
    for dep_name, dep_value in dependencies.items():
        dep_info = parse_dependency(dep_value)
        dep_info["name"] = dep_name
        
        if dep_info.get("type") == "workspace":
            workspace_deps.append(dep_info)
        elif dep_info.get("optional"):
            optional_deps.append(dep_info)
        else:
            external_deps.append(dep_info)
    
    dev_dependencies = cargo_data.get("dev-dependencies", {})
    if not isinstance(dev_dependencies, dict):
        dev_dependencies = {}
    
    dev_deps = []
    for dep_name, dep_value in dev_dependencies.items():
        dep_info = parse_dependency(dep_value)
        dep_info["name"] = dep_name
        dev_deps.append(dep_info)
    
    # Extract features
    features = cargo_data.get("features", {})
    if not isinstance(features, dict):
        features = {}
    
    default_features = features.get("default", [])
    if not isinstance(default_features, list):
        default_features = []
    
    optional_features = [f for f in features.keys() if f != "default"]
    
    # Find source files
    src_dir = crate_path / "src"
    source_files = []
    if src_dir.exists():
        for file in src_dir.rglob("*.rs"):
            rel_path = str(file.relative_to(crate_path))
            source_files.append(rel_path)
        source_files.sort()
    
    # Build metadata JSON-LD
    metadata = {
        "@context": relative_context_path(relative_path),
        "@type": "kotoba:Crate",
        "@id": f"kotoba:crate/{crate_name}",
        "name": crate_name,
        "version": version,
        "path": relative_path,
        "status": "active",
    }
    
    if edition:
        metadata["edition"] = edition
    if license:
        metadata["license"] = license
    if repository:
        metadata["repository"] = repository
    if description:
        metadata["description"] = description
    if keywords:
        metadata["keywords"] = keywords
    if categories:
        metadata["categories"] = categories
    
    if layer:
        metadata["layer"] = f"kotoba:layer/{layer}"
    
    metadata["dependencies"] = {
        "workspace": workspace_deps,
        "external": external_deps,
        "optional": optional_deps,
        "dev": dev_deps,
    }
    
    metadata["features"] = {
        "default": default_features,
        "optional": optional_features,
    }
    
    if source_files:
        metadata["sourceFiles"] = source_files
    
    return metadata

def main():
    """Main function to extract metadata from all crates."""
    workspace_root = Path(__file__).parent.parent
    crates_dir = workspace_root / "crates"
    
    if not crates_dir.exists():
        print(f"Error: {crates_dir} does not exist")
        return
    
    # Find all Cargo.toml files
    cargo_files = list(crates_dir.rglob("Cargo.toml"))
    
    print(f"Found {len(cargo_files)} Cargo.toml files")
    
    # Extract metadata for each crate
    for cargo_file in cargo_files:
        crate_path = cargo_file.parent
        metadata = extract_crate_metadata(crate_path, workspace_root)
        
        if metadata:
            # Save metadata.jsonld
            metadata_file = crate_path / "metadata.jsonld"
            with open(metadata_file, 'w', encoding='utf-8') as f:
                json.dump(metadata, f, indent=2, ensure_ascii=False)
            print(f"Generated: {metadata_file}")
        else:
            # Mark as unimplemented if no Cargo.toml or parse error
            metadata_file = crate_path / "metadata.jsonld"
            relative_path = str(crate_path.relative_to(workspace_root))
            layer = find_crate_layer(relative_path)
            
            unimplemented_metadata = {
                "@context": relative_context_path(relative_path),
                "@type": "kotoba:Crate",
                "@id": f"kotoba:crate/{crate_path.name}",
                "name": crate_path.name,
                "path": relative_path,
                "status": "unimplemented",
            }
            
            if layer:
                unimplemented_metadata["layer"] = f"kotoba:layer/{layer}"
            
            with open(metadata_file, 'w', encoding='utf-8') as f:
                json.dump(unimplemented_metadata, f, indent=2, ensure_ascii=False)
            print(f"Marked as unimplemented: {metadata_file}")

if __name__ == "__main__":
    main()

