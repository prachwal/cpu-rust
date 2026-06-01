# Apple-1 ROM Files

This directory is for storing ROM images for the Apple-1 computer.

## WOZ Monitor

The Apple-1 originally shipped with the **WOZ Monitor** (Woźniak Monitor) written by Steve Wozniak.
This is a 256-byte monitor program that provides basic functionality for examining and modifying memory,
running programs, and displaying memory contents.

### File: `wozmon.bin`

- **Size**: 256 bytes
- **Address Range**: $FF00-$FFFF (or $F000-$F0FF in some configurations)
- **Format**: Raw binary

### Legal Notice

The WOZ Monitor ROM is copyrighted software. The original source code is available from various
historical sources and has been disassembled and documented by the retrocomputing community.

### Obtaining the ROM

For legal reasons, the ROM file is not included in this repository. You can:

1. **Extract from an Apple-1 emulator**: Many Apple-1 emulators include the WOZ Monitor ROM.
2. **Assemble from source**: The WOZ Monitor source code is available online and can be assembled.
3. **Download from retrocomputing archives**: Various retrocomputing websites host the ROM.

### Source Code Reference

The original WOZ Monitor source code can be found at:
- [GitHub: jefftranter/6502 - wozmon.s](https://github.com/jefftranter/6502/blob/master/asm/wozmon/wozmon.s)
- [Steckschwein: WOZMON Analysis](https://www.steckschwein.de/post/wozmon-a-memory-monitor-in-256-bytes/)

### Creating the ROM File

If you have the source code, you can assemble it using a 6502 assembler:

```bash
# Using a 6502 assembler (e.g., acme, 64tass, or custom)
asm6502 wozmon.s wozmon.bin
```

Or extract from an existing Apple-1 emulator ROM image.

### Usage in Tests

For testing purposes, you can provide the ROM data through `ProfileLoadOptions`:

```csharp
var wozMonRom = File.ReadAllBytes("roms/apple-1/wozmon.bin");
var options = new ProfileLoadOptions(
    RomDataOverrides: new Dictionary<string, byte[]> {
        ["roms/apple-1/wozmon.bin"] = wozMonRom
    }
);

var computer = ComputerBuilder.BuildFromFile("profiles/computers/apple-1.json", options);
```

### Without ROM

The Apple-1 profile can work without the ROM file. The ROM region is marked as optional,
and the profile will still load (though the WOZ Monitor won't be functional).

For smoke tests that require the WOZ Monitor to run, the test should be marked as `[Explicit]`
or skipped if the ROM file is not available.
