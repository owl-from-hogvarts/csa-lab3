

# Регистры

- accumulator
- data (чтение запись из в память)
- state
- address
- program counter
- cmd(arg, opcode)


# Старт симуляции

Прочитать файл программы. Загрузить секции по указанным адресам. Загрузить данные в буфер ввода. 

# ISA

Так как команды загружаются из json файла, нет необходимости реализовывать регистр команд.

```
IN
OUT

LOAD address
STORE address

ADD
INC
AND (to check for even values by applying 0x1 mask)
ANDI
CMP
SHIFT_LEFT

JZC, JZ
JZS
JCS
JCC
JUMP

NOP
HALT
```

## Instruction format

<table>
    <thead>
        <tr>
            <th>opcode</th>
            <th>argument type</th>
            <th>argument</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td>1 byte</td>
            <td align="center">1 byte</td>
            <td>2 bytes</td>
        </tr>
        <tr>
            <td colspan=3 align="center" >4 bytes</td>
        </tr>
    </tbody>
</table>


## Instruction pipeline

1) Instruction fetch
   1) fetches instruction from memory to cmd register
2) operand decode
   1) determines type of operand and load it
3) execution
   1) determines microinstruction number by opcode. jumps there and execute

## Addressing modes (argument types)

- Absolute = operand -> address -> [mem] -> data
- Relative = pc + operand -> address -> [mem] -> data
- Indirect = pc + operand -> address -> [mem] -> data -> address -> data

# Status register

- C - Carry
- Z - Zero

# Memory

Memory is u32 addressable.



