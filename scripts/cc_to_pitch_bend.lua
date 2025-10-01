-- CC to Pitch Bend Converter
-- Converts CC messages on controller 80 to pitch bend messages
-- CC value 0-127 is mapped to pitch bend range -8192 to +8191

local CONTROL_CHANGE = 0xB0
local PITCH_BEND = 0xE0
local TARGET_CC = 80  -- Controller number to convert

local function get_message_type(status)
    return status & 0xF0
end

local function get_channel(status)
    return status & 0x0F
end

-- Convert CC value (0-127) to pitch bend value (-8192 to +8191)
-- Pitch bend is sent as 14-bit value split into LSB and MSB
local function cc_to_pitch_bend(cc_value)
    -- Map 0-127 to 0-16383 (14-bit range)
    -- 0 = -8192 (max down), 64 = 0 (center), 127 = +8191 (max up)
    local pitch_bend_14bit = math.floor((cc_value * 16383) / 127)
    
    -- Split into LSB (7 bits) and MSB (7 bits)
    local lsb = pitch_bend_14bit & 0x7F
    local msb = (pitch_bend_14bit >> 7) & 0x7F
    
    return lsb, msb
end

function process_midi(message)
    if #message < 3 then
        return {message}  -- Pass through short messages
    end
    
    local status = message[1]
    local msg_type = get_message_type(status)
    local channel = get_channel(status)
    
    -- Check if this is a CC message on controller 80
    if msg_type == CONTROL_CHANGE then
        local controller = message[2]
        local value = message[3]
        
        if controller == TARGET_CC then
            -- Convert to pitch bend message
            local lsb, msb = cc_to_pitch_bend(value)
            local pitch_bend_status = PITCH_BEND | channel
            
            return {{pitch_bend_status, lsb, msb}}
        end
    end
    
    -- Pass through all other messages unchanged
    return {message}
end

print("CC to Pitch Bend converter loaded")
print("Converting CC #" .. TARGET_CC .. " to pitch bend messages")
print("CC value mapping:")
print("  0   = -8192 (max down)")
print("  64  = 0 (center)")
print("  127 = +8191 (max up)")
