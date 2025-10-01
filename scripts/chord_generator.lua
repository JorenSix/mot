-- Chord Generator - Generates chords from single notes
-- This example creates major triads

local NOTE_ON = 0x90
local NOTE_OFF = 0x80

-- Chord intervals in semitones
local CHORD_INTERVALS = {0, 4, 7}  -- Major triad: root, major third, perfect fifth

local function get_message_type(status)
    return status & 0xF0
end

-- Store active notes to generate matching note-offs
local active_chords = {}

function process_midi(message)
    if #message < 3 then
        return {message}  -- Return single message in array
    end
    
    local status = message[1]
    local msg_type = get_message_type(status)
    local root_note = message[2]
    local velocity = message[3]
    
    if msg_type == NOTE_ON and velocity > 0 then
        -- Generate chord for note-on
        local chord_notes = {}
        
        for i, interval in ipairs(CHORD_INTERVALS) do
            local note = root_note + interval
            if note <= 127 then
                table.insert(chord_notes, note)
            end
        end
        
        -- Store chord for later note-off
        active_chords[root_note] = chord_notes
        
        -- Return multiple MIDI messages, one for each chord note
        local messages = {}
        for i, note in ipairs(chord_notes) do
            table.insert(messages, {status, note, velocity})
        end
        return messages
        
    elseif msg_type == NOTE_OFF or (msg_type == NOTE_ON and velocity == 0) then
        -- Generate note-off for the chord
        if active_chords[root_note] then
            local chord_notes = active_chords[root_note]
            active_chords[root_note] = nil
            
            -- Return note-off messages for all chord notes
            local messages = {}
            local note_off_status = (msg_type == NOTE_OFF) and status or (NOTE_OFF | (status & 0x0F))
            for i, note in ipairs(chord_notes) do
                table.insert(messages, {note_off_status, note, velocity})
            end
            return messages
        end
    end
    
    -- Pass through other messages in array format
    return {message}
end

print("Chord generator loaded - converts single notes to major triads")
print("Now generates multiple MIDI messages per input note!")
