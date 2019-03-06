# Debugging ASML Programs

To invoke the debugger, use the `DEBUG` instruction.

When the VM encounters a debug instruction, execution is paused and the
debug prompt is shown: `Debug> `.

## Commands

There are several commands available in the debugger:

- `step` - Step forward one instruction
- `memory`|`mem` - Dump memory
    - An address can be given to print the contents of a specific address:
    `mem 002B`.
- `continue`|`con` - Continue execution until the next debug instruction
- `disable`|`dis` - Disable any future debug instructions
- `enable`|`en` - Re-enable debugging (only useful right a `disable` and before `continue`)
- `next` - Print next instruction
- `registers`|`reg` - Print registers including pc and sp
- `printer`|`print` - Print contents of vm printer
- `exit`|`quit` - Exit application
