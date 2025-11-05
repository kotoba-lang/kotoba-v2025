//! WASM runtime for JSON-LD IR execution
//!
//! Provides functions to execute JSON-LD IRs in WebAssembly runtime.
//!
//! This module provides a basic WASM runtime interface for executing JSON-LD IRs.
//! The actual WASM modules that implement IR execution will be provided separately
//! (e.g., via fukurow WASM engine integration).

use serde_json::Value;
use anyhow::{Context, Result as AnyhowResult};

#[cfg(feature = "wasm")]
use wasmtime::*;

/// WASM runtime for IR execution
#[cfg(feature = "wasm")]
pub struct WasmRuntime {
    engine: Engine,
    module_cache: std::collections::HashMap<String, Module>,
}

#[cfg(feature = "wasm")]
impl WasmRuntime {
    /// Create a new WASM runtime
    pub fn new() -> AnyhowResult<Self> {
        let engine = Engine::default();
        Ok(Self {
            engine,
            module_cache: std::collections::HashMap::new(),
        })
    }

    /// Load a WASM module from bytes
    pub fn load_module(&mut self, name: &str, wasm_bytes: &[u8]) -> AnyhowResult<()> {
        let module = Module::new(&self.engine, wasm_bytes)
            .context(format!("Failed to compile WASM module: {}", name))?;
        self.module_cache.insert(name.to_string(), module);
        Ok(())
    }

    /// Execute Rule-IR in WASM
    pub fn execute_rule_jsonld(&self, rule_jsonld: &Value, module_name: &str) -> AnyhowResult<Value> {
        let module = self.module_cache.get(module_name)
            .ok_or_else(|| anyhow::anyhow!("WASM module not found: {}", module_name))?;

        let mut store = Store::new(&self.engine, ());
        let linker = Linker::new(&self.engine);
        let instance = linker.instantiate(&mut store, module)
            .context("Failed to instantiate WASM module")?;

        // Serialize rule_jsonld to JSON string
        let rule_json_str = serde_json::to_string(rule_jsonld)
            .context("Failed to serialize Rule-IR to JSON")?;

        // Call the execute_rule function in WASM module
        let execute_rule = instance.get_typed_func::<(&mut Store<()>, (i32, i32)), i32>(&mut store, "execute_rule")
            .context("Failed to get execute_rule function")?;

        // Allocate memory for input JSON string
        let memory = instance.get_memory(&mut store, "memory")
            .ok_or_else(|| anyhow::anyhow!("Memory export not found"))?;

        let input_ptr = allocate_string(&mut store, &memory, &rule_json_str)?;
        let input_len = rule_json_str.len() as i32;

        // Execute the rule
        let output_ptr = execute_rule.call(&mut store, (input_ptr, input_len))
            .context("Failed to execute rule in WASM")?;

        // Read output from WASM memory
        let output_json = read_string(&mut store, &memory, output_ptr as u32)
            .context("Failed to read output from WASM memory")?;

        // Parse output JSON to Value
        let result: Value = serde_json::from_str(&output_json)
            .context("Failed to parse WASM output as JSON")?;

        Ok(result)
    }

    /// Execute Query-IR in WASM
    pub fn execute_query_jsonld(&self, query_jsonld: &Value, module_name: &str) -> AnyhowResult<Value> {
        let module = self.module_cache.get(module_name)
            .ok_or_else(|| anyhow::anyhow!("WASM module not found: {}", module_name))?;

        let mut store = Store::new(&self.engine, ());
        let linker = Linker::new(&self.engine);
        let instance = linker.instantiate(&mut store, module)
            .context("Failed to instantiate WASM module")?;

        let query_json_str = serde_json::to_string(query_jsonld)
            .context("Failed to serialize Query-IR to JSON")?;

        let execute_query = instance.get_typed_func::<(&mut Store<()>, (i32, i32)), i32>(&mut store, "execute_query")
            .context("Failed to get execute_query function")?;

        let memory = instance.get_memory(&mut store, "memory")
            .ok_or_else(|| anyhow::anyhow!("Memory export not found"))?;

        let input_ptr = allocate_string(&mut store, &memory, &query_json_str)?;
        let input_len = query_json_str.len() as i32;

        let output_ptr = execute_query.call(&mut store, (input_ptr, input_len))
            .context("Failed to execute query in WASM")?;

        let output_json = read_string(&mut store, &memory, output_ptr as u32)
            .context("Failed to read output from WASM memory")?;

        let result: Value = serde_json::from_str(&output_json)
            .context("Failed to parse WASM output as JSON")?;

        Ok(result)
    }

    /// Execute Patch-IR in WASM
    pub fn execute_patch_jsonld(&self, patch_jsonld: &Value, module_name: &str) -> AnyhowResult<Value> {
        let module = self.module_cache.get(module_name)
            .ok_or_else(|| anyhow::anyhow!("WASM module not found: {}", module_name))?;

        let mut store = Store::new(&self.engine, ());
        let linker = Linker::new(&self.engine);
        let instance = linker.instantiate(&mut store, module)
            .context("Failed to instantiate WASM module")?;

        let patch_json_str = serde_json::to_string(patch_jsonld)
            .context("Failed to serialize Patch-IR to JSON")?;

        let execute_patch = instance.get_typed_func::<(&mut Store<()>, (i32, i32)), i32>(&mut store, "execute_patch")
            .context("Failed to get execute_patch function")?;

        let memory = instance.get_memory(&mut store, "memory")
            .ok_or_else(|| anyhow::anyhow!("Memory export not found"))?;

        let input_ptr = allocate_string(&mut store, &memory, &patch_json_str)?;
        let input_len = patch_json_str.len() as i32;

        let output_ptr = execute_patch.call(&mut store, (input_ptr, input_len))
            .context("Failed to execute patch in WASM")?;

        let output_json = read_string(&mut store, &memory, output_ptr as u32)
            .context("Failed to read output from WASM memory")?;

        let result: Value = serde_json::from_str(&output_json)
            .context("Failed to parse WASM output as JSON")?;

        Ok(result)
    }

    /// Execute Strategy-IR in WASM
    pub fn execute_strategy_jsonld(&self, strategy_jsonld: &Value, module_name: &str) -> AnyhowResult<Value> {
        let module = self.module_cache.get(module_name)
            .ok_or_else(|| anyhow::anyhow!("WASM module not found: {}", module_name))?;

        let mut store = Store::new(&self.engine, ());
        let linker = Linker::new(&self.engine);
        let instance = linker.instantiate(&mut store, module)
            .context("Failed to instantiate WASM module")?;

        let strategy_json_str = serde_json::to_string(strategy_jsonld)
            .context("Failed to serialize Strategy-IR to JSON")?;

        let execute_strategy = instance.get_typed_func::<(&mut Store<()>, (i32, i32)), i32>(&mut store, "execute_strategy")
            .context("Failed to get execute_strategy function")?;

        let memory = instance.get_memory(&mut store, "memory")
            .ok_or_else(|| anyhow::anyhow!("Memory export not found"))?;

        let input_ptr = allocate_string(&mut store, &memory, &strategy_json_str)?;
        let input_len = strategy_json_str.len() as i32;

        let output_ptr = execute_strategy.call(&mut store, (input_ptr, input_len))
            .context("Failed to execute strategy in WASM")?;

        let output_json = read_string(&mut store, &memory, output_ptr as u32)
            .context("Failed to read output from WASM memory")?;

        let result: Value = serde_json::from_str(&output_json)
            .context("Failed to parse WASM output as JSON")?;

        Ok(result)
    }
}

#[cfg(feature = "wasm")]
/// Allocate a string in WASM memory
fn allocate_string(store: &mut Store<()>, memory: &Memory, s: &str) -> AnyhowResult<i32> {
    // For now, return a placeholder. Actual implementation will depend on
    // the WASM module's memory allocation API
    // TODO: Implement proper memory allocation once WASM module interface is defined
    Err(anyhow::anyhow!("Memory allocation not yet implemented. WASM module interface needs to be defined."))
}

#[cfg(feature = "wasm")]
/// Read a string from WASM memory
fn read_string(store: &mut Store<()>, memory: &Memory, ptr: u32) -> AnyhowResult<String> {
    let mem = memory.data(store);
    let start = ptr as usize;
    
    if start >= mem.len() {
        return Err(anyhow::anyhow!("Invalid memory pointer: {}", ptr));
    }
    
    // Find null terminator or use length
    let mut len = 0;
    while start + len < mem.len() && mem[start + len] != 0 {
        len += 1;
    }

    let bytes = &mem[start..start + len];
    String::from_utf8(bytes.to_vec())
        .context("Failed to decode string from WASM memory")
}

/// Execute Rule-IR in WASM (convenience function)
#[cfg(feature = "wasm")]
pub fn execute_rule_jsonld(rule_jsonld: &Value, module_name: &str, wasm_bytes: &[u8]) -> AnyhowResult<Value> {
    let mut runtime = WasmRuntime::new()?;
    runtime.load_module(module_name, wasm_bytes)?;
    runtime.execute_rule_jsonld(rule_jsonld, module_name)
}

/// Execute Query-IR in WASM (convenience function)
#[cfg(feature = "wasm")]
pub fn execute_query_jsonld(query_jsonld: &Value, module_name: &str, wasm_bytes: &[u8]) -> AnyhowResult<Value> {
    let mut runtime = WasmRuntime::new()?;
    runtime.load_module(module_name, wasm_bytes)?;
    runtime.execute_query_jsonld(query_jsonld, module_name)
}

/// Execute Patch-IR in WASM (convenience function)
#[cfg(feature = "wasm")]
pub fn execute_patch_jsonld(patch_jsonld: &Value, module_name: &str, wasm_bytes: &[u8]) -> AnyhowResult<Value> {
    let mut runtime = WasmRuntime::new()?;
    runtime.load_module(module_name, wasm_bytes)?;
    runtime.execute_patch_jsonld(patch_jsonld, module_name)
}

/// Execute Strategy-IR in WASM (convenience function)
#[cfg(feature = "wasm")]
pub fn execute_strategy_jsonld(strategy_jsonld: &Value, module_name: &str, wasm_bytes: &[u8]) -> AnyhowResult<Value> {
    let mut runtime = WasmRuntime::new()?;
    runtime.load_module(module_name, wasm_bytes)?;
    runtime.execute_strategy_jsonld(strategy_jsonld, module_name)
}

