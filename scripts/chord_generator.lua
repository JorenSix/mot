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
        return message
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
        
        -- Return just the root note (other notes would need separate MIDI messages)
        -- In a real implementation, you'd send multiple messages
        -- For now, return the first chord note
        if #chord_notes > 0 then
            return {status, chord_notes[1], velocity}
        end
        
    elseif msg_type == NOTE_OFF or (msg_type == NOTE_ON and velocity == 0) then
        -- Generate note-off for the chord
        if active_chords[root_note] then
            local chord_notes = active_chords[root_note]
            active_chords[root_note] = nil
            
            if #chord_notes > 0 then
                -- Return note-off for first chord note
                local note_off_status = (msg_type == NOTE_OFF) and status or (NOTE_OFF | (status & 0x0F))
                return {note_off_status, chord_notes[1], velocity}
            end
        end
    end
    
    -- Pass through other messages
    return message
end

print("Chord generator loaded - converts single notes to major triads")
print("Note: Due to MIDI message structure, this sends only the root note.")
print("For full chords, you'd need to generate multiple MIDI messages per input.")
