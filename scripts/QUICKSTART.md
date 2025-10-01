# Quick Start Guide - Lua MIDI Processor

## Installation

1. Build the project:
```bash
cd /Users/jansix/Projects/mot
cargo build --release
```

## 5-Minute Tutorial

### Step 1: Check Available MIDI Devices
```bash
./target/release/mot lua_process -l
```

You'll see something like:
```
Available MIDI input ports:
0: Your MIDI Device
6666: Virtual mot input port

Available MIDI output ports:
0: Your MIDI Device  
6666: Virtual mot ouput port
```

### Step 2: Test with Passthrough

Create or use the included `passthrough.lua`:
```lua
function process_midi(message)
    return message  -- Pass all messages through unchanged
end
```

Run it:
```bash
./target/release/mot lua_process \
  --midi_input_index 0 \
  --midi_output_index 0 \
  --script passthrough.lua \
  -v
```

The `-v` flag shows you what's happening in real-time.

### Step 3: Try Note Transposition

Use `example_processor.lua` to transpose notes up one octave:

```bash
./target/release/mot lua_process \
  --midi_input_index 0 \
  --midi_output_index 0 \
  --script example_processor.lua \
  -v
```

Play some notes and hear them transposed!

### Step 4: Write Your Own Script

Create `my_script.lua`:
```lua
-- Double the velocity of all notes
local NOTE_ON = 0x90

function process_midi(message)
    local msg_type = message[1] & 0xF0
    
    if msg_type == NOTE_ON and #message >= 3 then
        local velocity = math.min(message[3] * 2, 127)
        return {message[1], message[2], velocity}
    end
    
    return message
end
```

Run it:
```bash
./target/release/mot lua_process \
  --midi_input_index 0 \
  --midi_output_index 0 \
  --script my_script.lua \
  -v
```

## Common Patterns

### Filter by MIDI Channel
```lua
function process_midi(message)
    local channel = message[1] & 0x0F
    if channel == 0 then
        return message  -- Only pass channel 0
    else
        return nil      -- Block everything else
    end
end
```

### Transpose Notes
```lua
function process_midi(message)
    local NOTE_ON = 0x90
    local NOTE_OFF = 0x80
    local msg_type = message[1] & 0xF0
    
    if (msg_type == NOTE_ON or msg_type == NOTE_OFF) and #message >= 3 then
        local new_note = message[2] + 12  -- Up one octave
        if new_note <= 127 then
            return {message[1], new_note, message[3]}
        end
    end
    return message
end
```

### Scale Velocity
```lua
function process_midi(message)
    local NOTE_ON = 0x90
    local msg_type = message[1] & 0xF0
    
    if msg_type == NOTE_ON and #message >= 3 then
        local scaled = math.floor(message[3] * 0.7)  -- 70% velocity
        return {message[1], message[2], scaled}
    end
    return message
end
```

## Tips

1. **Start Simple** - Begin with passthrough, then add features
2. **Use Verbose Mode** - The `-v` flag helps debug
3. **Test with Virtual Ports** - Use index 6666 for testing without hardware
4. **Return nil to Filter** - Block unwanted messages
5. **Check Your Lua** - Test syntax with `lua myscript.lua` first

## What's in the Message?

MIDI messages are tables of bytes:
- `message[1]` - Status (type + channel)
- `message[2]` - Data 1 (e.g., note number)
- `message[3]` - Data 2 (e.g., velocity)

Status byte:
- Upper 4 bits: 0x90 = Note On, 0x80 = Note Off, 0xB0 = CC, etc.
- Lower 4 bits: MIDI channel (0-15)

Extract them:
```lua
local msg_type = message[1] & 0xF0  -- Get type
local channel = message[1] & 0x0F    -- Get channel
```

## Need Help?

1. Check `LUA_PROCESSOR_README.md` for detailed docs
2. Look at the example scripts
3. Use verbose mode to see what's happening
4. The original mot README for general MIDI/OSC info

## Example Scripts Included

- `passthrough.lua` - No processing
- `example_processor.lua` - Transpose + examples
- `velocity_filter.lua` - Velocity gate
- `channel_router.lua` - Channel mapper
- `arpeggiator.lua` - Simple arp
- `chord_generator.lua` - Chord creator

Try them all!

Happy MIDI processing! 🎹
