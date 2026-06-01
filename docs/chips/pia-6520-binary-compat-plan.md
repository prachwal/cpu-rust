# MOS 6520 / Motorola 6821 PIA binary compatibility plan

Goal: make `crates/chips/pia-6520` behave like real 6520/6821-compatible PIA at the register, pin, interrupt, and bus-visible side-effect level.

Current state: minimal port/DDRx/control model. It is enough for simple stubs, not binary-compatible hardware emulation.

## Compatibility target

- Model 6821 PIA first; document any MOS 6520 differences before enabling them.
- Preserve `no_std`.
- Keep bus API deterministic and testable: register access plus explicit pin-line methods.
- Expose observable external pins for machine crates, without embedding Apple/PET-specific behavior.

## Public model required

- Registers:
  - `ORA`, `ORB`
  - `DDRA`, `DDRB`
  - `CRA`, `CRB`
  - input/output latch state
  - CA1, CA2, CB1, CB2 input/output line state
  - IRQ A and IRQ B output state
- Register select behavior:
  - address 0 selects `ORA` or `DDRA` depending on `CRA bit 2`
  - address 2 selects `ORB` or `DDRB` depending on `CRB bit 2`
  - address 1 is `CRA`
  - address 3 is `CRB`
- Read/write methods:
  - `read(addr) -> u8`
  - `write(addr, value)`
  - external pin setters, e.g. `set_port_a_pins`, `set_ca1`, `set_ca2`, `set_cb1`, `set_cb2`
  - external output getters, e.g. `port_a_pins`, `port_b_pins`, `ca2_output`, `cb2_output`, `irq_a`, `irq_b`

## Register behavior to implement

- Port data reads:
  - DDR bit `1`: read output latch/driven value
  - DDR bit `0`: read external pin/input latch according to selected latch mode
  - verify exact behavior for input latch vs live pin on 6821
- Port data writes:
  - if control bit selects DDR, write DDR
  - if control bit selects OR, write output latch
  - output pins update only where DDR bit is `1`
- Control register bits:
  - `CRA bit 0`: CA1 active transition select
  - `CRA bit 1`: CA2 active transition/select control low bit
  - `CRA bit 2`: port A data direction select
  - `CRA bits 3..5`: CA2 mode
  - `CRA bit 6`: CA1 interrupt flag
  - `CRA bit 7`: CA2 interrupt flag
  - mirror same semantics for `CRB` with CB1/CB2
- Interrupt flags:
  - set on selected CA1/CB1 transitions
  - set on selected CA2/CB2 transitions when configured as input
  - clear on documented register reads/writes only
  - preserve flag bits correctly when reading control registers
  - `IRQ_A` asserts when enabled flag is pending
  - `IRQ_B` asserts when enabled flag is pending
- CA2/CB2 modes:
  - input interrupt modes
  - output manual low/high modes
  - pulse output mode
  - handshake output mode
  - restore behavior after port reads/writes
- Reset behavior:
  - all registers, latches, flags, and line states match hardware reset defaults
  - IRQ outputs inactive
  - output pins high impedance where DDR is `0`

## Timing and side effects

- Define whether PIA is cycle-aware or access-event-aware.
- If cycle-aware:
  - add `tick(cycles)` for pulse-width/handshake timing
  - test single-cycle pulse behavior
- If access-event-aware:
  - document approximation and blocked compatibility gaps
- Reads that clear flags must do so in the exact documented order.
- Writes that affect CA2/CB2 output must update pins before returning.

## Test coverage required

- Register select tests:
  - ORA vs DDRA selected by `CRA bit 2`
  - ORB vs DDRB selected by `CRB bit 2`
  - control register read masks and flag bits
- Port data tests:
  - all DDR masks: `0x00`, `0xFF`, alternating masks, single-bit masks
  - live input pin changes visible on input bits
  - output latch preserved while DDR changes
  - output pin result changes after DDR writes
- Interrupt tests:
  - CA1 rising edge
  - CA1 falling edge
  - CB1 rising edge
  - CB1 falling edge
  - CA2 input rising/falling modes
  - CB2 input rising/falling modes
  - disabled interrupt flag pending without IRQ assertion
  - enabled interrupt flag asserting IRQ
  - exact clear-on-read behavior
- Handshake tests:
  - CA2 pulse mode
  - CB2 pulse mode
  - CA2 handshake mode
  - CB2 handshake mode
  - output pin transitions on port access
- Reset tests:
  - reset after arbitrary dirty state
  - reset line state and IRQ state
- Machine regression tests:
  - Apple 1 keyboard ready/data behavior still works
  - PET keyboard PIA scan still works after moving to true PIA semantics

## Reference material needed

- Motorola MC6821 datasheet.
- MOS MCS6520 datasheet.
- Existing Apple 1 and PET behavior tests in sibling projects.
- Any available PIA exerciser ROMs or minimal bus test programs.

## Implementation order

1. Replace direct public register mutation with explicit state and accessors.
2. Implement register select and DDR/OR behavior.
3. Add CA1/CB1 edge detection and IRQ flags.
4. Add CA2/CB2 input interrupt modes.
5. Add CA2/CB2 output, pulse, and handshake modes.
6. Add reset and pin-state APIs.
7. Port Apple 1/PET machine code to the stricter API.
8. Add exhaustive unit tests for every control mode and flag transition.
9. Add machine-level regression tests.

## Done criteria

- Every documented control-register mode has a unit test.
- Every interrupt source has enabled, disabled, set, clear, and IRQ-output tests.
- Every register read/write side effect is tested.
- Machine crates no longer rely on public mutation of PIA internals.
- Remaining deviations from real 6520/6821 behavior are documented as explicit unsupported quirks.
