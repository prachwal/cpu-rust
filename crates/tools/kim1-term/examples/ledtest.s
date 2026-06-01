; KIM-1 Direct LED Test
; Writes directly to RIOT 6530-002 ports to drive the 7-segment LEDs.
; PA = segment pattern, PB = digit select (active low)
; Segment bits: bit0=A, bit1=B, bit2=C, bit3=D, bit4=E, bit5=F, bit6=G

    .org $0200

start:
    ldx #$00          ; digit counter (0-5)

next_digit:
    lda digit_table,x ; get pattern for current digit
    sta $1C00         ; write to 6530-002 PA (segments)

    lda select_table,x ; get bit mask for this digit (active low)
    sta $1C01         ; write to 6530-002 PB (digit select)

    ldy #$20          ; small delay
delay:
    dey
    bne delay

    inx
    cpx #$06
    bne next_digit
    beq start

; Segment patterns (A=bit0 ... G=bit6)
; 0=ABCDEF, 1=BC, 2=ABDEG, 3=ABCDG, 4=BCFG, 5=ACDFG
digit_table:
    .byte $3F         ; 0: ABCDEF
    .byte $06         ; 1: BC
    .byte $5B         ; 2: ABDEG
    .byte $4F         ; 3: ABCD G
    .byte $66         ; 4: BCFG
    .byte $6D         ; 5: ACDFG

select_table:
    .byte $FE         ; digit 0 (bit 0 = 0)
    .byte $FD         ; digit 1 (bit 1 = 0)
    .byte $FB         ; digit 2 (bit 2 = 0)
    .byte $F7         ; digit 3 (bit 3 = 0)
    .byte $EF         ; digit 4 (bit 4 = 0)
    .byte $DF         ; digit 5 (bit 5 = 0)
