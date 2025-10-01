-- Example Lua MIDI Processor Script
-- This script demonstrates various MIDI processing techniques

-- Define constants for MIDI message types
local NOTE_OFF = 0x80
local NOTE_ON = 0x90
local CONTROL_CHANGE = 0xB0
local PROGRAM_CHANGE = 0xC0

-- Helper function to get MIDI message type
local function get_message_type(status)
    return status & 0xF0
end

-- Helper function to get MIDI channel (0-15)
local function get_channel(status)
    return status & 0x0F
end

-- Main processing function
-- This function receives a MIDI message as a table of bytes
-- Return nil to filter (block) the message
-- Return a table of bytes to send the processed message
function process_midi(message)
    -- Ensure we have at least one byte
    if #message == 0 then
        return nil
    end
    
    local status = message[1]
    local msg_type = get_message_type(status)
    local channel = get_channel(status)
    
    -- Example 1: Pass through all messages unchanged
    -- return message
    
    -- Example 2: Transpose notes up by one octave
    if msg_type == NOTE_ON or msg_type == NOTE_OFF then
        if #message >= 3 then
            local note = message[2]
            local velocity = message[3]
            
            -- Transpose up by 12 semitones (one octave)
            local new_note = note + 12
            
            -- Make sure note is in valid MIDI range (0-127)
            if new_note <= 127 then
                return {status, new_note, velocity}
            else
                -- Filter out notes that would go out of range
                return nil
            end
        end
    end
    
    -- Example 3: Filter by channel (only pass channel 0)
    -- if channel == 0 then
    --     return message
    -- else
    --     return nil
    -- end
    
    -- Example 4: Remap MIDI channels (shift all messages from channel 0 to channel 1)
    -- if channel == 0 then
    --     local new_status = (status & 0xF0) | 1  -- Keep message type, set channel to 1
    --     message[1] = new_status
    --     return message
    -- end
    
    -- Example 5: Filter out note-off messages, pass everything else
    -- if msg_type == NOTE_OFF then
    --     return nil
    -- else
    --     return message
    -- end
    
    -- For all other messages, pass through unchanged
    return message
end

-- Optional: Print script info when loaded
print("Lua MIDI processor loaded!")
print("Current configuration: Transpose notes up by one octave")
