-- Channel router - route messages from one channel to another
-- Example: route all messages from channel 0 to channel 5

local SOURCE_CHANNEL = 0
local DEST_CHANNEL = 5

local function get_channel(status)
    return status & 0x0F
end

function process_midi(message)
    if #message == 0 then
        return nil
    end
    
    local status = message[1]
    local channel = get_channel(status)
    
    -- If message is on source channel, remap to destination channel
    if channel == SOURCE_CHANNEL then
        -- Keep message type, replace channel
        local new_status = (status & 0xF0) | DEST_CHANNEL
        message[1] = new_status
    end
    
    return message
end

print(string.format("Channel router loaded - mapping channel %d to channel %d", 
    SOURCE_CHANNEL, DEST_CHANNEL))
