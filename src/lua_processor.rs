use mlua::prelude::*;
use std::fs;

pub struct LuaProcessor {
    lua: Lua,
}

impl LuaProcessor {
    /// Create a new Lua processor and load a script file
    pub fn new(script_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let lua = Lua::new();
        
        // Read the Lua script
        let script_content = fs::read_to_string(script_path)?;
        
        // Execute the script to load functions
        lua.load(&script_content).exec()?;
        
        Ok(LuaProcessor { lua })
    }
    
    /// Process a MIDI message through the Lua script
    /// Returns the processed MIDI message, or None if the message should be filtered
    pub fn process_message(&self, message: &[u8]) -> LuaResult<Option<Vec<u8>>> {
        // Get the process_midi function from Lua
        let process_fn: LuaFunction = self.lua.globals().get("process_midi")?;
        
        // Convert message to Lua table
        let message_table = self.lua.create_table()?;
        for (i, &byte) in message.iter().enumerate() {
            message_table.set(i + 1, byte)?; // Lua uses 1-based indexing
        }
        
        // Call the Lua function with the message
        let result: LuaValue = process_fn.call(message_table)?;
        
        // Handle the result
        match result {
            LuaValue::Nil => Ok(None), // Filter the message
            LuaValue::Table(table) => {
                // Convert Lua table back to Vec<u8>
                let mut output = Vec::new();
                for pair in table.pairs::<usize, u8>() {
                    let (_key, value) = pair?;
                    output.push(value);
                }
                Ok(Some(output))
            }
            _ => Err(LuaError::RuntimeError(
                "process_midi must return nil or a table of bytes".to_string(),
            )),
        }
    }
}
