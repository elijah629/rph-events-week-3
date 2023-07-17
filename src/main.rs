/// Matrix3D or M3D aka M3 is the worlds best file format for formatting formats to a format formatting format.
///
/// How it works:
///
/// Serialize
/// [ stuff ] -> [ magic ] -> [ different stuff ]
///
/// Deserialize
/// [ different stuff ] -> [ magic ] -> [ stuff ]
///
/// There is support for a file signature (M3) but it is not enabled in this demo for file size concerns
/// This uses ZLib to compress files after my raw serialization
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use chrono::prelude::*;
use enum_as_inner::EnumAsInner;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::{self, BufRead, Cursor, Read, Write};
use std::path::Path;
use std::{env, fs};

fn main() {
    // Path
    let output = input("Enter the output file's name").unwrap();
    let path = env::current_dir().unwrap().join(output);
    let path = path.as_path();

    // Generates the binary and write to file
    generate(vec![vec![vec![7]]], &path).unwrap();

    // Reads the file and return the file
    let file = read(&path).unwrap();

    // Print output as per the challange requirementss
    println!("--- Decoded");
    println!("Author: {}", file.header.name);
    println!(
        "Created: {:?}",
        Utc.timestamp_opt(file.header.time as i64, 0).unwrap()
    );
    println!("Data: {:?}", file.data);

    let raw = fs::read(&path).unwrap();
    println!("File size: {} bytes", raw.len())
}

type Matrix3D<T> = Vec<Vec<Vec<T>>>;

#[derive(Debug)]
struct Header {
    // signature: [u8; 2],
    time: u64,
    name: String,
}

#[derive(Debug)]
struct File {
    header: Header,
    data: Matrix3D<i32>,
}

#[derive(Debug)]
enum OpCode {
    /// Data layout:
    /// ```_
    /// len   value
    /// u64 [u8; len] // Total opcode length in bytes
    /// ```
    Vec,
    /// Data layout:
    /// ```_
    /// len   value
    /// u64 [u8; len]
    /// ```
    Slice,
    /// Data layout:
    /// ```_
    /// value
    /// i32
    /// ```
    I32,
    /// Data layout:
    /// ```_
    /// value
    /// u64
    /// ```
    U64,
}

impl Into<u8> for OpCode {
    fn into(self) -> u8 {
        match self {
            OpCode::Vec => 0x00,
            OpCode::Slice => 0x01,
            OpCode::I32 => 0x10,
            OpCode::U64 => 0x11,
        }
    }
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        match value {
            0x00 => OpCode::Vec,
            0x01 => OpCode::Slice,
            0x10 => OpCode::I32,
            0x11 => OpCode::U64,
            _ => panic!("Impossible..."),
        }
    }
}

#[derive(Debug, EnumAsInner, Clone)]
enum Value {
    I32(i32),
    U64(u64),
    String(String),
    Vec(Vec<Value>),
}

struct Serializer {
    values: Vec<Value>,
}

impl Serializer {
    pub fn add(&mut self, value: Value) {
        self.values.push(value);
    }

    pub fn serialize(self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Self::serialize_values(self.values)
    }

    fn serialize_values(values: Vec<Value>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut w: Cursor<Vec<u8>> = Cursor::new(vec![]);

        for value in values {
            let w: &mut Cursor<Vec<u8>> = &mut w;
            match value {
                Value::I32(x) => {
                    w.write_u8(OpCode::I32.into())?;
                    w.write_i32::<BigEndian>(x)?;
                }
                Value::U64(x) => {
                    w.write_u8(OpCode::U64.into())?;
                    w.write_u64::<BigEndian>(x)?;
                }
                Value::String(x) => {
                    w.write_u8(OpCode::Slice.into())?;
                    w.write_u64::<BigEndian>(x.len() as u64)?;
                    w.write_all(x.as_bytes())?;
                }
                Value::Vec(x) => {
                    let x = Self::serialize_values(x)?;

                    w.write_u8(OpCode::Vec.into())?;
                    w.write_u64::<BigEndian>(x.len() as u64)?;
                    w.write_all(x.as_slice())?;
                }
            }
        }

        Ok(w.into_inner())
    }
    pub fn new() -> Self {
        Self { values: vec![] }
    }
}

struct Deserializer {
    buffer: Vec<u8>,
}

impl<'a> Deserializer {
    pub fn deserialize(self) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        Self::deserialize_values(self.buffer)
    }

    fn deserialize_values(buffer: Vec<u8>) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let mut w: Cursor<Vec<u8>> = Cursor::new(buffer);
        let mut values: Vec<Value> = vec![];

        // let pos = w.position();
        while !w.fill_buf()?.is_empty() {
            let opcode: OpCode = w.read_u8()?.into();
            match opcode {
                OpCode::Vec => {
                    let len = w.read_u64::<BigEndian>()? as usize;
                    let mut buf = vec![0; len];
                    w.read_exact(&mut buf)?;
                    values.push(Value::Vec(Self::deserialize_values(buf)?));
                }
                OpCode::Slice => {
                    let len = w.read_u64::<BigEndian>()? as usize;
                    let mut buf = vec![0; len];
                    w.read_exact(&mut buf)?;
                    values.push(Value::String(String::from_utf8(buf)?));
                }
                OpCode::I32 => {
                    let x = w.read_i32::<BigEndian>()?;
                    values.push(Value::I32(x));
                }
                OpCode::U64 => {
                    let x = w.read_u64::<BigEndian>()?;
                    values.push(Value::U64(x));
                }
            }
        }

        Ok(values)
    }

    pub fn new(buffer: Vec<u8>) -> Self {
        Self { buffer }
    }
}

fn generate(matrix: Matrix3D<i32>, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let time = Utc::now().timestamp() as u64;

    let name = input("Enter your name")?;

    let header = Header {
        // signature: *b"M3",
        time,
        name,
    };
    let file = File {
        header,
        data: matrix,
    };

    let mut serializer = Serializer::new();

    // serializer.add(Value::String(
    //     String::from_utf8_lossy(&file.header.signature).to_string(),
    // ));

    serializer.add(Value::U64(file.header.time));
    serializer.add(Value::String(file.header.name));

    fn matrix3d_to_value(matrix: Matrix3D<i32>) -> Value {
        Value::Vec(
            matrix
                .iter()
                .map(|a| {
                    Value::Vec(
                        a.iter()
                            .map(|b| {
                                Value::Vec(b.iter().map(|&c| Value::I32(c)).collect::<Vec<_>>())
                            })
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>(),
        )
    }

    serializer.add(matrix3d_to_value(file.data));

    let serialized = serializer.serialize()?;

    let mut compressor = ZlibEncoder::new(vec![], Compression::best());
    compressor.write_all(&serialized)?;
    let compressed = compressor.finish()?;

    fs::write(&output_path, &compressed)?;

    Ok(())
}

fn read(path: &Path) -> Result<File, Box<dyn std::error::Error>> {
    let compressed = fs::read(&path)?;
    let mut decompresser = ZlibDecoder::new(compressed.as_slice());
    let mut decompressed = vec![];
    decompresser.read_to_end(&mut decompressed).unwrap();

    let mut deserialized = Deserializer::new(decompressed).deserialize()?.into_iter();

    let header = Header {
        // signature: deserialized
        //     .next()
        //     .unwrap()
        //     .as_string()
        //     .unwrap()
        //     .as_bytes()
        //     .try_into()
        //     .unwrap(),
        time: deserialized.next().unwrap().into_u64().unwrap(),
        name: deserialized.next().unwrap().into_string().unwrap(),
    };

    fn value_to_matrix3d(value: Value) -> Matrix3D<i32> {
        value
            .into_vec()
            .unwrap()
            .into_iter()
            .map(|x| {
                x.into_vec()
                    .unwrap()
                    .into_iter()
                    .map(|x| {
                        x.into_vec()
                            .unwrap()
                            .into_iter()
                            .map(|x| x.into_i32().unwrap())
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    }

    let file = File {
        header,
        data: value_to_matrix3d(deserialized.next().unwrap()),
    };

    Ok(file)
}

fn input(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    print!("{}: ", prompt);
    io::stdout().flush()?;

    let buf = &mut String::new();
    io::stdin().read_line(buf)?;

    Ok(buf.trim_end().to_owned())
}
