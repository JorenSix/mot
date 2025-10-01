-- Test script to verify Lua processor is working correctly
-- This script prints diagnostic information and processes messages

local message_count = 0
local note_on_count = 0
local note_off_count = 0

local NOTE_ON = 0x90
local NOTE_OFF = 0x80
local CONTROL_CHANGE = 0xB0

local function get_message_type(status)
    return status & 0xF0
end

local function get_channel(status)
    return status & 0x0F
end

local function message_type_name(msg_type)
    if msg_type == NOTE_ON then return "Note On"
    elseif msg_type == NOTE_OFF then return "Note Off"
    elseif msg_type == CONTROL_CHANGE then return "CC"
    else return string.format("0x%02X", msg_type)
    end
end

function process_midi(message)
    if #message == 0 then
        print("Warning: Empty message received")
        return nil
    end
    
    message_count = message_count + 1
    
    local status = message[1]
    local msg_type = get_message_type(status)
    local channel = get_channel(status)
    
    -- Count message types
    if msg_type == NOTE_ON and #message >= 3 and message[3] > 0 then
        note_on_count = note_on_count + 1
    elseif msg_type == NOTE_OFF or (msg_type == NOTE_ON and #message >= 3 and message[3] == 0) then
        note_off_count = note_off_count + 1
    end
    
    -- Print detailed info every 10 messages
    if message_count % 10 == 0 then
        print(string.format("--- Statistics: %d messages (%d note-on, %d note-off) ---", 
            message_count, note_on_count, note_off_count))
    end
    
    -- Print message details
    if #message >= 3 then
        print(string.format("Msg #%d: %s ch:%d data:[%d, %d]", 
            message_count,
            message_type_name(msg_type),
            channel,
            message[2],
            message[3]))
    else
        print(string.format("Msg #%d: Type:0x%02X (short message)", 
            message_count, status))
    end
    
    -- Pass through all messages unchanged
    return message
end

print("==============================================")
print("Lua MIDI Test Script Loaded Successfully!")
print("==============================================")
print("This script will:")
print("  - Count all MIDI messages")
print("  - Track note-on and note-off messages")
print("  - Print statistics every 10 messages")
print("  - Pass all messages through unchanged")
print("")
print("Send some MIDI to test!")
print("==============================================")
