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
    /// Returns a vector of processed MIDI messages (can be empty, single, or multiple messages)
    pub fn process_message(&self, message: &[u8]) -> LuaResult<Vec<Vec<u8>>> {
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
            LuaValue::Nil => Ok(Vec::new()), // Filter the message (return empty array)
            LuaValue::Table(table) => {
                // Check if this is an array of messages or a single message
                // If first element is a number, it's a single MIDI message
                // If first element is a table, it's an array of messages
                
                let first_value: LuaValue = table.get(1)?;
                
                match first_value {
                    LuaValue::Integer(_) | LuaValue::Number(_) => {
                        // Single MIDI message: {status, data1, data2, ...}
                        let mut output = Vec::new();
                        for pair in table.pairs::<usize, u8>() {
                            let (_key, value) = pair?;
                            output.push(value);
                        }
                        Ok(vec![output])
                    }
                    LuaValue::Table(_) => {
                        // Array of MIDI messages: {{status, data1, data2}, {status, data1, data2}, ...}
                        let mut messages = Vec::new();
                        for pair in table.pairs::<usize, LuaTable>() {
                            let (_key, msg_table) = pair?;
                            let mut output = Vec::new();
                            for msg_pair in msg_table.pairs::<usize, u8>() {
                                let (_msg_key, value) = msg_pair?;
                                output.push(value);
                            }
                            messages.push(output);
                        }
                        Ok(messages)
                    }
                    LuaValue::Nil => {
                        // Empty table, return empty array
                        Ok(Vec::new())
                    }
                    _ => Err(LuaError::RuntimeError(
                        "process_midi must return nil, a table of bytes, or an array of byte tables".to_string(),
                    )),
                }
            }
            _ => Err(LuaError::RuntimeError(
                "process_midi must return nil, a table of bytes, or an array of byte tables".to_string(),
            )),
        }
    }
}
