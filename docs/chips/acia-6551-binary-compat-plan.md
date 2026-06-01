# MOS 6551 ACIA binary compatibility plan

Goal: make `crates/chips/acia-6551` behave like real MOS 6551 Asynchronous
Communications Interface Adapter at the register, status, interrupt, and
serial data level.

## Register map (4 addresses)

| Addr&3 | Read           | Write          |
|--------|----------------|----------------|
| 0      | Receive Data   | Transmit Data  |
| 1      | Status         | Reset (any)    |
| 2      | Command Reg    | Command Reg    |
| 3      | Control Reg    | Control Reg    |

## Status Register (read at base+1)

```
bit 7: IRQ (1 = interrupt pending)
bit 6: DSR (Data Set Ready — 1 = ready)
bit 5: DCD (Data Carrier Detect — 1 = detected)
bit 4: Tx Empty (1 = transmit buffer empty)
bit 3: Rx Full (1 = receive data full)
bit 2: Overrun error
bit 1: Framing error
bit 0: Parity error
```

Writing any value to base+1 performs a software reset.

## Command Register (base+2)

```
bits 7-6: Parity
  00 = disabled
  01 = odd
  10 = even
  11 = mark (transmit parity bit always 1)
bit 5: Parity type select (0 = normal, 1 = mark/space)
bit 4: Receiver echo mode (1 = echo, 0 = normal)
bit 3: Tx IRQ enable (1 = enabled)
bit 2: Rx IRQ enable (1 = enabled)
bits 1-0: Mode
  00 = normal
  01 = echo (Rx data looped to Tx)
  10 = local loopback (TxD=high, RxD connected to TxD)
  11 = remote loopback (TxD = RxD)
```

## Control Register (base+3)

```
bits 7-5: Baud rate
  000 = 16× external clock
  001 = 50
  010 = 75
  011 = 109.92
  100 = 134.58
  101 = 150
  110 = 300
  111 = 600
bits 4-2: Baud rate (continued)
  000 = 1200
  001 = 1800
  010 = 2400
  011 = 3600
  100 = 4800
  101 = 7200
  110 = 9600
  111 = 19200
bit 1: Word length (0 = 7 bits, 1 = 8 bits)
bit 0: Number of stop bits (0 = 1, 1 = 2)
```

## Implementation

- `Acia6551` struct: registers, internal state, rx/tx buffers
- `read(addr)`: returns register value with proper status bits
- `write(addr, val)`: writes to register
- `transmit(data)`: push byte to external output
- `receive(data)`: external input → internal rx buffer
- `rx_ready()`, `tx_ready()`: status checks
- Timer callback for baud-based serial timing

## Test coverage

- Register read/write
- Reset clears status
- Tx buffer empty flag set after write
- Rx buffer full flag after receive
- Parity generation/checking
- Overrun/framing/parity error flags
- Loopback modes
- IRQ output behavior
- All baud rates via timer callback
