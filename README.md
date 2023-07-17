# M3 Docs

M3 is an efficent binary format for compressing and storing 3D matricies.

## How it works

There are 4 data types,

- **I32**  
  Stores a `i32`
- **U64**  
  Stores a `u64`
- **String**  
  Stores a `String`
- **Vec**  
  Stores a `Vec<Value>`

Each data type has a respective opcode type. An opcode is a byte followed by optional arguments. There are 4 opcodes.

- **Vec** `0x00`

  ```_
  len   value
  u64 [u8; len]
  ```

  `len` is the total length of the sub-opcodes in bytes

- **Slice** `0x01`

  ```_
  len   value
  u64 [u8; len]
  ```

- **I32** `0x10`

  ```_
  value
  i32
  ```

- **U64** `0x11`

  ```_
  value
  u64
  ```

> The codeblock below each type is the data layout, ie the opcode followed by these arguments, first row is the header

The values get converted to opcodes and thier arguments, then the bytes get written to a buffer. The buffer is then ZLib compressed and written to disk.
