-- Simple passthrough - forwards all MIDI messages unchanged
-- This is the simplest possible processor

function process_midi(message)
    -- Return the message as-is in an array format
    return {message}
end

print("Passthrough processor loaded - all MIDI messages will be forwarded unchanged")
