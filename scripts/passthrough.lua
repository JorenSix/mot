-- Simple passthrough - forwards all MIDI messages unchanged
-- This is the simplest possible processor

function process_midi(message)
    -- Just return the message as-is
    return message
end

print("Passthrough processor loaded - all MIDI messages will be forwarded unchanged")
