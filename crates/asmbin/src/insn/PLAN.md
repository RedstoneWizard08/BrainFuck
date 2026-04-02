# REX Prefix

Needed when any of:

```rs
operands.any(|it| it.bit_width == 64) && !insn.default_operand_size_64bit
```

```rs
operands.any(|it| (R8..R15 + XMM8..XMM15 + YMM8..YMM15 + CR8..CR15 + DR8..DR15).contains(it))
```

```rs
operands.any(|it| [SPL, BPL, SIL, DIL].contains(it))
```

And not:

```rs
operands.any(|it| [AH, CH, BH, DH].contains(it))
```

## Instructions defaulting to 64-bit operand size in long mode

- CALL (near)
- ENTER
- Jcc
- JrCXZ
- JMP (near)
- LEAVE
- LGDT
- LIDT
- LLDT
- LOOP
- LOOPcc
- LTR
- MOV CR(n)
- MOV DR(n)
- POP reg/mem
- POP reg
- POP FS
- POP GS
- POPFQ
- PUSH imm8
- PUSH imm32
- PUSH reg/mem
- PUSH reg
- PUSH FS
- PUSH GS
- PUSHFQ
- RET (near)
