-- Arpeggiator - Generates arpeggios from held notes
-- Note: This is a simplified version for demonstration
-- A full arpeggiator would need timing/clock handling

local NOTE_ON = 0x90
local NOTE_OFF = 0x80

-- Store held notes
local held_notes = {}
local arp_index = 0

local function get_message_type(status)
    return status & 0xF0
end

local function add_note(note)
    -- Add note if not already in list
    for i, n in ipairs(held_notes) do
        if n == note then
            return
        end
    end
    table.insert(held_notes, note)
    table.sort(held_notes)
end

local function remove_note(note)
    for i, n in ipairs(held_notes) do
        if n == note then
            table.remove(held_notes, i)
            return
        end
    end
end

function process_midi(message)
    if #message < 3 then
        return message
    end
    
    local status = message[1]
    local msg_type = get_message_type(status)
    local note = message[2]
    local velocity = message[3]
    
    if msg_type == NOTE_ON and velocity > 0 then
        -- Note on: add to held notes
        add_note(note)
        
        -- Generate arpeggio pattern (up)
        if #held_notes > 0 then
            arp_index = (arp_index % #held_notes) + 1
            local arp_note = held_notes[arp_index]
            return {status, arp_note, velocity}
        end
        
        return nil  -- Filter original note
        
    elseif msg_type == NOTE_OFF or (msg_type == NOTE_ON and velocity == 0) then
        -- Note off: remove from held notes
        remove_note(note)
        
        -- Reset arp index if no notes held
        if #held_notes == 0 then
            arp_index = 0
        end
        
        return nil  -- Filter note-off messages
    end
    
    -- Pass through other messages
    return message
end

print("Arpeggiator loaded - plays held notes in sequence")
print("Note: This is a simple demonstration. Real arpeggiators need clock sync.")
