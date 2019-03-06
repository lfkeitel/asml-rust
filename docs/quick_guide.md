# Instruction Quick Guide

This table lists all instructions and their binary formats as well as a short
description of the instructions. More information can be found in the
[reference guide](reference_guide.md).

| Instruction | Binary Formats                                        | Desc                                                                         |
|-------------|-------------------------------------------------------|------------------------------------------------------------------------------|
| ADD         | ADD: 01 %R HH LL<br>IMM: 02 %R HH LL<br>REG: 03 %R %R | Add two registers                                                            |
| AND         | ADD: 04 %R HH LL<br>IMM: 05 %R HH LL<br>REG: 06 %R %R | Bitwise and two registers                                                    |
| CALL        | ADD: 0F HH LL<br>REG: 10 %R                           | Make a subroutine call to an address                                         |
| HALT        | INH: 12                                               | Halt execution                                                               |
| JMP         | REG: 13 %R HH LL                                      | Jump execution to a different address if register equals value in register 0 |
| JMPA        | ADD: 14 HH LL                                         | Jump execution to a different address always                                 |
| LDSP        | ADD: 15 %R HH LL<br>IMM: 16 %R HH LL<br>REG: 17 %R %R | Load stack pointer                                                           |
| LOAD        | ADD: 18 %R HH LL<br>IMM: 19 %R HH LL<br>REG: 1A %R %R | Load data from memory into a register                                        |
| NOOP        | INH: 00                                               | Do nothing for one cycle                                                     |
| OR          | ADD: 07 %R HH LL<br>IMM: 08 %R HH LL<br>REG: 09 %R %R | Bitwise or two registers                                                     |
| POP         | REG: 1E %R                                            | Pop data from software stack                                                 |
| PUSH        | REG: 1F %R                                            | Push data to software stack                                                  |
| ROTL        | REG: 0E %R NN                                         | Rotate data in a register left                                               |
| ROTR        | REG: 0D %R NN                                         | Rotate data in a register right                                              |
| RTN         | INH: 11                                               | Return from a subroutine call                                                |
| STR         | ADD: 1B %R HH LL<br>REG: 1C %R %R                     | Store data from a register to memory                                         |
| XFER        | REG: 1D %R %R                                         | Move data between registers                                                  |
| XOR         | ADD: 0A %R HH LL<br>IMM: 0B %R HH LL<br>REG: 0C %R %R | Bitwise xor two registers                                                    |

Mode Key:

- `ADD` - Address mode
- `IMM` - Immediate mode
- `REG` - Register mode
- `INH` - Inherent mode

Format Key:

- Literal values are in hex
- `%R` - Register
- `HH` - High byte of address or immediate value
- `LL` - Low byte of address or immediate value
- `NN` - Single byte number
