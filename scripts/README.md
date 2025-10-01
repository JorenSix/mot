# Lua MIDI Processor

Process MIDI messages in real-time using Lua scripts!

```
┌─────────────────┐
│  MIDI Input     │
│  Device/Port    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  MidiIn         │
│  .listen()      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Lua Processor  │
│  process_midi() │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  MidiOut        │
│  .send_full()   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  MIDI Output    │
│  Device/Port    │
└─────────────────┘
```


## Quick Start

1. Build the application:
   ```bash
   cargo build --release
   ```

2. List available MIDI devices:
   ```bash
   ./target/release/mot lua_process -l
   ```

3. Run with a Lua script:
   ```bash
   ./target/release/mot lua_process \
     --midi_input_index 0 \
     --midi_output_index 6666 \
     --script example_processor.lua \
     -v
   ```

## Usage

```bash
mot lua_process [OPTIONS] --script <SCRIPT>
```

### Options

- `-s, --script <SCRIPT>` - Path to the Lua script file (required)
- `--midi_input_index <INDEX>` - MIDI input device index (default: 0)
- `--midi_output_index <INDEX>` - MIDI output device index (default: 0)
- `-v` - Verbose mode (print debug information)
- `-l` - List available MIDI devices

### Special Device Indices

- `6666` - Creates a virtual MIDI port named "mot virtual port" (Unix only)

## Writing Lua Scripts

Your Lua script must define a `process_midi` function that:
- Takes a table of bytes (the MIDI message)
- Returns a table of bytes (the processed message) OR `nil` to filter the message

### Basic Template

```lua
function process_midi(message)
    -- Your processing logic here
    return message  -- Return modified message or nil to filter
end
```

### MIDI Message Structure

MIDI messages are tables of bytes:

- `message[1]` - Status byte (message type + channel)
- `message[2]` - First data byte (e.g., note number)
- `message[3]` - Second data byte (e.g., velocity)

The status byte contains:
- Upper 4 bits: Message type (0x80 = Note Off, 0x90 = Note On, etc.)
- Lower 4 bits: MIDI channel (0-15)

### Examples

#### 1. Passthrough (no processing)
```lua
function process_midi(message)
    return message
end
```

#### 2. Transpose notes up by one octave
```lua
local NOTE_ON = 0x90
local NOTE_OFF = 0x80

function process_midi(message)
    local status = message[1]
    local msg_type = status & 0xF0
    
    if (msg_type == NOTE_ON or msg_type == NOTE_OFF) and #message >= 3 then
        local note = message[2] + 12  -- Transpose up by 12 semitones
        if note <= 127 then
            return {status, note, message[3]}
        end
    end
    
    return message
end
```

#### 3. Filter messages by channel
```lua
function process_midi(message)
    local channel = message[1] & 0x0F
    
    if channel == 0 then
        return message  -- Pass only channel 0
    else
        return nil      -- Filter all other channels
    end
end
```

#### 4. Velocity scaling
```lua
local NOTE_ON = 0x90

## Building and Running

### Build
```bash
cd /Users/jansix/Projects/mot
cargo build --release
```

### Run with examples
```bash
# List MIDI devices
./target/release/mot lua_process -l

# Basic passthrough
./target/release/mot lua_process \
  --midi_input_index 0 \
  --midi_output_index 0 \
  --script passthrough.lua

# Transpose with virtual output port
./target/release/mot lua_process \
  --midi_input_index 0 \
  --midi_output_index 6666 \
  --script example_processor.lua \
  -v

# Filter by velocity
./target/release/mot lua_process \
  --midi_input_index 0 \
  --midi_output_index 0 \
  --script velocity_filter.lua

# Route channels
./target/release/mot lua_process \
  --midi_input_index 0 \
  --midi_output_index 0 \
  --script channel_router.lua
```

## Troubleshooting

### Script not loading
- Check that the file path is correct
- Ensure the script has `process_midi` function defined
- Check syntax with `lua script.lua` first

### No MIDI output
- Verify output device index with `-l` flag
- Check that script returns a message (not nil)
- Use `-v` flag to see what's happening

### Performance issues
- Keep Lua functions simple and fast
- Avoid heavy computations in the message handler
- Consider filtering early to reduce processing

## Contributing

Feel free to share your Lua MIDI processing scripts!
