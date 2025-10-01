-- Velocity filter - only pass through notes with velocity above a threshold
-- Useful for filtering out quiet notes or creating a velocity gate

local VELOCITY_THRESHOLD = 64  -- Only pass notes with velocity > 64

local NOTE_OFF = 0x80
local NOTE_ON = 0x90

local function get_message_type(status)
    return status & 0xF0
end

function process_midi(message)
    if #message == 0 then
        return nil
    end
    
    local status = message[1]
    local msg_type = get_message_type(status)
    
    -- For note messages, check velocity
    if (msg_type == NOTE_ON or msg_type == NOTE_OFF) and #message >= 3 then
        local velocity = message[3]
        
        -- Filter out notes below threshold
        if velocity <= VELOCITY_THRESHOLD then
            return nil
        end
    end
    
    -- Pass through the message
    return message
end

print("Velocity filter loaded - threshold: " .. VELOCITY_THRESHOLD)
