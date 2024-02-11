# Implementation P0

- [x] [OAM DM Transfer](https://gbdev.io/pandocs/OAM_DMA_Transfer.html#oam-dma-transfer)
- [x] [LCD Y Coordinate](https://gbdev.io/pandocs/STAT.html#ff44--ly-lcd-y-coordinate-read-only)
- [x] [LY Compare](https://gbdev.io/pandocs/STAT.html#ff45--lyc-ly-compare)
- [x] [STAT](https://gbdev.io/pandocs/STAT.html#ff41--stat-lcd-status)
- [x] [VBlank Interrupt](https://gbdev.io/pandocs/Interrupt_Sources.html#int-40--vblank-interrupt)
- [x] [STAT Interrupt](https://gbdev.io/pandocs/Interrupt_Sources.html#int-48--stat-interrupt)
- [ ] [MBC](https://gbdev.io/pandocs/MBCs.html)

# Implementation P1
- [ ] [Mode 3 penalties](https://gbdev.io/pandocs/Rendering.html#mode-3-length)
- [ ] [Joypad](https://gbdev.io/pandocs/Joypad_Input.html#joypad-input)
- [ ] [Joypad Interrupt](https://gbdev.io/pandocs/Interrupt_Sources.html#int-60--joypad-interrupt)
- [ ] [Reset DIV timer on write](https://gbdev.io/pandocs/Timer_and_Divider_Registers.html#ff04--div-divider-register)

# Implementation P2

- [ ] [Return 0 from FEA0-FEFF range](https://gbdev.io/pandocs/Memory_Map.html#fea0-feff-range)
- [ ] [LCD disable](https://gbdev.io/pandocs/LCDC.html#lcdc7--lcd-enable)
- [ ] [Audio](https://gbdev.io/pandocs/Audio.html#audio-overview)
- [ ] Handle GPU interrupts before next instruction

# Architecture

- [ ] Unify `MemoryMappedDevice`
- [x] Extract functions in `tick_mode3`
- [x] Make more functions private in `gb.rs`
- [ ] Make GPU buffer full frame with colors rather than sending back pixels
