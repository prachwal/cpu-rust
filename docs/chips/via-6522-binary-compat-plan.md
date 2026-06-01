# MOS 6522 VIA binary compatibility plan

Goal: make `crates/chips/via-6522` behave like real MOS 6522 VIA at the register, pin, interrupt, timer, shift-register, and bus-visible side-effect level.

Current state: minimal register array, simplified timers, simplified IFR/IER, simple CA1/CB1 triggers. It is not binary-compatible.

## Compatibility target

- Model MOS 6522 first.
- Track NMOS 6522 behavior separately from WDC 65C22 differences.
- Preserve `no_std`.
- Make cycle timing deterministic through `tick(cycles)`.
- Keep machine-specific wiring outside the chip crate.

## Public model required

- Programmer-visible registers:
  - `ORB/IRB`
  - `ORA/IRA`
  - `DDRB`
  - `DDRA`
  - `T1C-L`, `T1C-H`
  - `T1L-L`, `T1L-H`
  - `T2C-L`, `T2C-H`
  - `SR`
  - `ACR`
  - `PCR`
  - `IFR`
  - `IER`
  - alternate `ORA no handshake`
- External pins:
  - PA0..PA7
  - PB0..PB7
  - CA1, CA2
  - CB1, CB2
  - IRQ
- Internal state:
  - port output latches
  - port input latches
  - timer counters/latches
  - one-shot fired state
  - PB7 timer output state
  - shift register state and bit counter
  - interrupt pending/enabled flags

## Register behavior to implement

- Ports:
  - DDR-masked reads and writes
  - ORA/ORB output latch semantics
  - IRA/IRB input latch semantics controlled by `ACR`
  - port read side effects on CA1/CA2/CB1/CB2 flags
  - register `$0F` ORA read/write without handshake side effects
- `ACR`:
  - bit 0: PA latch enable
  - bit 1: PB latch enable
  - bits 2..4: shift register mode
  - bit 5: Timer 2 pulse-count mode vs timed interrupt mode
  - bit 6: Timer 1 free-run vs one-shot
  - bit 7: PB7 output enable from Timer 1
- `PCR`:
  - CA1 active edge select
  - CA2 input edge modes
  - CA2 independent interrupt input
  - CA2 handshake output
  - CA2 pulse output
  - CA2 manual low/high output
  - CB1 active edge select
  - CB2 input edge modes
  - CB2 independent interrupt input
  - CB2 handshake output
  - CB2 pulse output
  - CB2 manual low/high output
- `IFR`:
  - bit 7 computed from `(IFR & IER & 0x7F) != 0`
  - writes clear selected bits, not ignored
  - reads do not clear all flags
  - source-specific clears happen only on documented register access
- `IER`:
  - bit 7 selects set/clear
  - read returns bit 7 set
  - bits 0..6 preserve enabled state

## Timer 1 requirements

- 16-bit latch and counter behavior.
- Low-byte write updates latch low only.
- High-byte write updates latch high, loads counter, clears IFR T1, starts timer.
- Read counter low/high returns current counter bytes.
- Reading high byte clears T1 IFR.
- One-shot mode:
  - underflow sets IFR T1 once until restarted
  - counter continues decrement/wrap behavior as real chip requires
- Free-run mode:
  - underflow sets IFR T1
  - reload from latch at correct cycle
  - repeated interrupts at correct period
- PB7 output mode:
  - disabled mode leaves PB7 controlled by ORB/DDRB
  - one-shot PB7 behavior
  - free-run PB7 toggling behavior
- Off-by-one timing:
  - verify exact delay from write high byte to first underflow
  - verify reload cycle and visible counter values around underflow

## Timer 2 requirements

- Timed interrupt mode:
  - low-byte write updates latch low
  - high-byte write loads counter and clears IFR T2
  - underflow sets IFR T2 once
  - read high byte clears IFR T2
- Pulse-count mode:
  - PB6 transition decrements counter
  - selected edge behavior documented and tested
  - underflow and flag behavior match hardware
- Counter read behavior:
  - low/high reads around underflow
  - latch/counter distinction

## Shift register requirements

- Implement all `ACR bits 2..4` modes:
  - disabled
  - shift in under T2 control
  - shift in under PHI2/system clock
  - shift in under external CB1 clock
  - shift out free-running under T2
  - shift out under T2 control
  - shift out under PHI2/system clock
  - shift out under external CB1 clock
- CB1 clock line behavior.
- CB2 data line behavior.
- bit count and IFR SR set after 8 shifts.
- read/write SR side effects.
- interrupt enable/disable interactions.

## CA/CB line requirements

- CA1/CB1:
  - rising/falling edge selection
  - edge detection based on previous pin state
  - flag set only on selected transition
  - clear-on-port-read behavior
- CA2/CB2:
  - input interrupt modes
  - independent interrupt modes
  - handshake output modes
  - pulse output modes
  - manual output low/high modes
  - exact clear behavior via port access
- IRQ output:
  - updates after every IFR/IER/source transition
  - active state exposed consistently to machines

## Test coverage required

- Register map:
  - every address `$0..$F` read/write behavior
  - reset values
  - unsupported writes ignored or handled exactly
- Ports:
  - all DDR masks
  - input latch enabled/disabled
  - ORA `$1` vs ORA `$F` handshake differences
  - PB7 timer output vs normal port output
- IFR/IER:
  - write-to-clear every flag bit
  - disabled pending flags do not assert IRQ
  - enabled pending flags assert IRQ
  - bit 7 computed behavior
  - source-specific clear behavior
- Timer 1:
  - one-shot underflow timing
  - one-shot restart
  - free-run repeated period
  - latch write combinations
  - reads around underflow
  - PB7 output modes
- Timer 2:
  - timed mode underflow
  - restart
  - pulse-count mode via PB6
  - reads around underflow
- Shift register:
  - every ACR shift mode
  - IFR SR set after exactly 8 bits
  - CB1/CB2 pin states
  - read/write side effects
- CA/CB lines:
  - every PCR mode
  - rising/falling edge select
  - handshake transitions
  - pulse duration
- Machine regression:
  - PET boot reaches READY
  - PET BASIC accepts host input
  - PET display update still works

## Reference material needed

- MOS Technology 6522 VIA datasheet.
- WDC 65C22 datasheet for documented deviations only.
- Known 6522 timer test ROMs or minimal 6502 bus exercisers.
- VICE or py65-style reference traces for VIA register/timing cases.
- PET boot/input tests from this repo and sibling projects.

## Implementation order

1. Split raw `regs[16]` into named fields and internal latches.
2. Implement correct IFR/IER semantics first.
3. Implement port DDR/latch behavior and ORA `$F` no-handshake access.
4. Implement CA1/CB1 edge detection.
5. Implement CA2/CB2 PCR modes.
6. Rework Timer 1 with exact one-shot/free-run/PB7 behavior.
7. Rework Timer 2 with timed and pulse-count modes.
8. Implement shift register modes.
9. Replace PET-specific shortcuts with proper line wiring where practical.
10. Add exhaustive unit tests and PET e2e regression tests.

## Done criteria

- Every documented register bit has tests for set, clear, read, write, and side effects.
- Every interrupt source has pending/enabled/disabled/clear/IRQ-output tests.
- Timer tests cover off-by-one boundaries around load, underflow, reload, and reads.
- Shift register tests cover all modes.
- PET terminal smoke test still passes without chip-specific hacks.
- Any non-NMOS-6522 behavior is behind an explicit quirk/config flag.
