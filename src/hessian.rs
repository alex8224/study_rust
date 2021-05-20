use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::{read, write};
use std::ops::RangeInclusive;
use std::time::{Duration, SystemTime};

use nom::bytes::complete::take;
use nom::Err;
use nom::error::{ErrorKind, make_error};
use nom::number::complete::{be_f64, be_i32, be_i64, be_u32, be_u64, be_u8, u16};

use chrono::{DateTime, Utc};

type ParseErr<'a> = Err<(&'a [u8], ErrorKind)>;

struct TypeRef<'a> {
    type_ref: HashMap<&'a str, i32>,
    buff: RefCell<&'a [u8]>,
}

impl<'a> TypeRef<'a> {
    fn new(out_buff: &'a [u8]) -> Self {
        Self {
            type_ref: HashMap::<&str, i32>::new(),
            buff: RefCell::new(out_buff),
        }
    }

    fn get(&self, key: &'a str) -> Option<&i32> {
        self.type_ref.get(key)
    }

    fn put(&mut self, key: &'a str, val: i32) {
        self.type_ref.insert(key, val);
    }

    fn parse_byte_chunks(&self) -> Result<&[u8], ParseErr> {
        let (i, h) = be_u8(*self.buff.borrow())?;
        let (i, l) = be_u8(i)?;
        let len = (u32::from(h) << 8) + u32::from(l);
        let (i, val) = take(len as usize)(i)?;
        self.buff.replace(i);
        Ok(val)
    }
    fn read_binary_bytag(&self, tag: u8) -> Result<Vec<u8>, ParseErr> {
        let mut offset = *self.buff.borrow();
        let mut last_chunk = false;
        let mut byte_buff = Vec::new();
        match tag {
            0x20..=0x2f => {
                //<=15
                let len = tag - 0x20;
                let (i, val) = take(len as usize)(offset)?;
                copy_slice(val, &mut byte_buff);
                self.buff.replace(i);
                Ok(byte_buff)
            }
            0x34..=0x37 => {
                // <=1023
                let (i, tag2) = be_u8(offset)?;
                let len = u32::from(tag - 0x34) * 256 + u32::from(tag2);
                let (i, val) = take(len as usize)(i)?;
                copy_slice(val, &mut byte_buff);
                self.buff.replace(i);
                Ok(byte_buff)
            }
            0x41..=0x42 => {
                last_chunk = tag == 0x42;
                loop {
                    let val = self.parse_byte_chunks()?;
                    copy_slice(val, &mut byte_buff);
                    offset = *self.buff.borrow();
                    if last_chunk {
                        break;
                    } else {
                        let (i, tag) = be_u8(offset)?;
                        offset = i;
                        self.buff.replace(offset);
                        last_chunk = tag == 0x42;
                    }
                }
                self.buff.replace(offset);
                Ok(byte_buff)
            }
            _ => Err(Err::Error(make_error(offset, ErrorKind::Eof)))
        }
    }

    fn read_binary(&mut self) -> Result<Vec<u8>, ParseErr> {
        let (i, tag) = be_u8(*self.buff.borrow())?;
        self.buff.replace(i);
        self.read_binary_bytag(tag)
    }

    fn read_bool(&self) -> Result<(&[u8], u8), ParseErr> {
        let (i, tag) = be_u8(*self.buff.borrow())?;
        Ok((i, tag))
    }
}

#[derive(Debug)]
pub enum ListValue {
    Empty,
    Str(String),
    Double(f64),
    Integer(i32),
    Long(i64),
    Date(u64),
    Boolean(bool),
    Byte(Vec<u8>),
    List(Vec<ListValue>),
}

#[derive(Debug)]
pub enum List {
    Empty,
    UTyped(Vec<Object>),
    Typed(String, Vec<Object>),
}

macro_rules! imp_for_list {
    () => {};
    ($($enum_val:ident -> $type_val:ty),*) => {
            $(
        impl From<$type_val> for ListValue {
            fn from(val: $type_val) -> Self {
                ListValue::$enum_val(val)
            }
        })*
    }
}

// imp_for_list![
//     Str -> String,
//     Double -> f64,
//     Integer -> i32,
//     Long-> i64,
//     Date -> u64,
//     Boolean -> bool,
//     Byte -> Vec<u8>,
//     List -> Vec<ListValue>
// ];

#[derive(Debug)]
pub enum Object {
    NULL,
    Boolean(bool),
    Integer(i32),
    Long(i64),
    Double(f64),
    Date(u64),
    Bin(Vec<u8>),
    Str(String),
    List(List),
}

pub fn parse_byte_chunks(input: &[u8]) -> Result<(&[u8], &[u8]), ParseErr> {
    let (i, h) = be_u8(input)?;
    let (i, l) = be_u8(i)?;
    let len = (u32::from(h) << 8) + u32::from(l);
    let (i, val) = take(len as usize)(i)?;
    Ok((i, val))
}

pub fn copy_slice(slice: &[u8], target: &mut Vec<u8>) {
    for idx in 0..slice.len() {
        target.push(slice[idx]);
    }
}

pub fn read_binary_bytag(i: &[u8], tag: u8) -> Result<(&[u8], Vec<u8>), ParseErr> {
    let mut offset = i;
    let mut last_chunk = false;
    let mut byte_buff = Vec::new();
    match tag {
        0x20..=0x2f => {
            //<=15
            let len = tag - 0x20;
            let (i, val) = take(len as usize)(i)?;
            copy_slice(val, &mut byte_buff);
            Ok((i, byte_buff))
        }
        0x34..=0x37 => {
            // <=1023
            let (i, tag2) = be_u8(i)?;
            let len = u32::from(tag - 0x34) * 256 + u32::from(tag2);
            let (i, val) = take(len as usize)(i)?;
            copy_slice(val, &mut byte_buff);
            offset = i;
            Ok((offset, byte_buff))
        }
        0x41..=0x42 => {
            last_chunk = tag == 0x42;
            loop {
                let (i, val) = parse_byte_chunks(offset)?;
                copy_slice(val, &mut byte_buff);
                offset = i;
                if last_chunk {
                    break;
                } else {
                    let (i, tag) = be_u8(offset)?;
                    offset = i;
                    last_chunk = tag == 0x42;
                }
            }
            Ok((offset, byte_buff))
        }
        _ => Err(Err::Error(make_error(i, ErrorKind::Eof)))
    }
}

pub fn read_binary(input: &[u8]) -> Result<(&[u8], Vec<u8>), ParseErr> {
    let (i, tag) = be_u8(input)?;
    read_binary_bytag(i, tag)
}

pub fn parse_utf8(input: &[u8]) -> Result<(&[u8], u32), ParseErr> {
    let (i, u8_tag) = be_u8(input)?;
    let mut offset = i;
    return if u8_tag < 0x80 {
        Ok((i, u8_tag as u32))
    } else if (u8_tag & 0xe0) == 0xc0 {
        let (i, chr2) = be_u8(i)?;
        offset = i;
        let ch = (u32::from(u8_tag & 0x1f) << 6) + u32::from(chr2 & 0x3f);
        Ok((offset, ch))
    } else {
        let (i, chr2) = be_u8(i)?;
        let (i, chr3) = be_u8(i)?;
        offset = i;
        let ch = (u32::from(u8_tag & 0x0f) << 12) + (u32::from(chr2 & 0x3f) << 6) + u32::from(chr3 & 0x3f);
        Ok((offset, ch))
    };
}

pub fn merge_char(input: &[u8], len: usize) -> Result<(&[u8], String), ParseErr> {
    let mut offset = input;
    let mut str_val = String::with_capacity(len * 3);
    for _ in 0..len {
        let (i, chr) = parse_utf8(offset)?;
        let chr = std::char::from_u32(chr).unwrap();
        str_val.push(chr);
        offset = i;
    }
    Ok((offset, str_val))
}

pub fn read_string_bytag(i: &[u8], tag: u8) -> Result<(&[u8], String), ParseErr> {
    let mut offset = i;
    let mut last_chunk = false;

    match tag {
        0x0..=0x1f => {
            let len = tag - 0x0;
            let (i, str_val) = merge_char(i, len as usize)?;
            Ok((i, str_val))
        }
        0x30..=0x33 => {
            let (i, tag2) = be_u8(i)?;
            let len = u32::from(tag - 0x30) * 256 + u32::from(tag2);
            let (i, str_val) = merge_char(offset, len as usize)?;
            offset = i;
            Ok((offset, str_val))
        }
        0x52..=0x53 => {
            last_chunk = tag == 0x53;
            let mut str_buff = String::new();
            loop {
                let (i, h) = be_u8(offset)?;
                let (i, l) = be_u8(i)?;
                // offset = i;
                let len = (u32::from(h) << 8) + u32::from(l);
                let (i, str_val) = merge_char(i, len as usize)?;
                offset = i;
                str_buff.push_str(str_val.as_str());
                if last_chunk {
                    break;
                } else {
                    let (i, tag) = be_u8(offset)?;
                    last_chunk = tag == 0x53;
                    offset = i;
                }
            }
            Ok((offset, str_buff))
        }
        _ => Err(Err::Error(make_error(i, ErrorKind::Eof)))
    }
}

pub fn read_string(input: &[u8]) -> Result<(&[u8], String), ParseErr> {
    let (i, tag) = be_u8(input)?;
    read_string_bytag(i, tag)
}

pub fn read_bool(input: &[u8]) -> Result<(&[u8], bool), ParseErr> {
    let (i, tag) = be_u8(input)?;
    if tag == 0x54 {
        Ok((i, true))
    } else {
        Ok((i, false))
    }
}

pub fn read_int_bytag(input: &[u8], tag: u8) -> Result<(&[u8], i32), ParseErr> {
    match tag {
        0x80..=0xbf => {
            Ok((input, (tag - 0x90) as i32))
        }
        0xc0..=0xcf => {
            //byte int
            let (i, l) = be_u8(input)?;
            let val = (i32::from(tag - 0xc8) << 8) + i32::from(l);
            Ok((i, val))
        }
        0xd0..=0xd7 => {
            //short
            let (i, h) = be_u8(input)?;
            let (i, l) = be_u8(i)?;
            let byte1 = (i32::from(tag) - 0xd4) << 16;
            let val = byte1 + 256 as i32 * i32::from(h) + i32::from(l);
            Ok((i, val))
        }
        0x49 | 0x59 => {
            //int
            let (i, val) = be_i32(input)?;
            Ok((i, val))
        }
        _ => Err(Err::Error(make_error(input, ErrorKind::Eof)))
    }
}

pub fn read_int(input: &[u8]) -> Result<(&[u8], i32), ParseErr> {
    let (i, tag) = be_u8(input)?;
    read_int_bytag(i, tag)
}


pub fn read_long_bytag(input: &[u8], tag: u8) -> Result<(&[u8], i64), ParseErr> {
    match tag {
        0xd8..=0xef => {
            Ok((input, i64::from(tag - 0xe0)))
        }
        0xf0..=0xff => {
            let (i, l) = be_u8(input)?;
            let val = ((i64::from(tag) - 0xf8) << 8) + i64::from(l);
            Ok((i, val))
        }
        0x38..=0x3f => {
            let (i, h) = be_u8(input)?;
            let (i, l) = be_u8(i)?;
            let val = ((i64::from(tag) - 0x3c) << 16) + 256 * i64::from(h) + i64::from(l);
            Ok((i, val))
        }
        0x49 | 0x59 => {
            let (i, val) = be_i32(input)?;
            Ok((i, val as i64))
        }
        0x4c => {
            let (i, val) = be_i64(input)?;
            Ok((i, val))
        }

        _ => Err(Err::Error(make_error(input, ErrorKind::Eof)))
    }
}

pub fn read_long(input: &[u8]) -> Result<(&[u8], i64), ParseErr> {
    let (i, tag) = be_u8(input)?;
    read_long_bytag(i, tag)
}

fn read_utcdate_bytag(i: &[u8], tag: u8) -> Result<(&[u8], u64), ParseErr> {
    match tag {
        0x4a => {
            // tag = J
            let (i, utc) = be_u64(i)?;
            Ok((i, utc))
        }
        0x4b => {
            // tag = K
            let (i, int) = be_u32(i)?;
            let utc = int as u64 * 60000;
            Ok((i, utc))
        }
        _ => Err(Err::Error(make_error(i, ErrorKind::Eof)))
    }
}

fn read_utcdate(input: &[u8]) -> Result<(&[u8], u64), ParseErr> {
    let (i, tag) = be_u8(input)?;
    read_utcdate_bytag(i, tag)
}

fn read_double_bytag(i: &[u8], tag: u8) -> Result<(&[u8], f64), ParseErr> {
    match tag {
        0x5b => {
            Ok((i, 0 as f64))
        }
        0x5c => {
            Ok((i, 1 as f64))
        }
        0x5d => {
            let (i, val) = be_u8(i)?;
            Ok((i, val as f64))
        }
        0x5e => {
            let (i, h) = be_u8(i)?;
            let (i, l) = be_u8(i)?;
            let val = u32::from(h) * 256 + u32::from(l);
            Ok((i, val as f64))
        }
        0x5f => {
            let (i, int) = be_i32(i)?;
            Ok((i, 0.001 * int as f64))
        }
        0x44 => {
            // tag = D
            let (i, double) = be_f64(i)?;
            Ok((i, double))
        }
        _ => Err(Err::Error(make_error(i, ErrorKind::Eof)))
    }
}

pub fn read_double(input: &[u8]) -> Result<(&[u8], f64), ParseErr> {
    let (i, tag) = be_u8(input)?;
    read_double_bytag(i, tag)
}

pub fn read_type(input: &[u8]) -> Result<(&[u8], String), ParseErr> {
    let (i, tag) = be_u8(input)?;
    match tag {
        0x0..=0x1f | 0x30..=0x33 | 0x52..=0x53 => {
            let (i, val) = read_string(input)?;
            Ok((i, val))
        }
        _ =>
            Err(Err::Error(make_error(input, ErrorKind::Eof)))
    }
}


pub fn read_object(input: &[u8]) -> Result<(&[u8], Object), ParseErr> {
    let (i, tag) = be_u8(input)?;
    let val = match tag {
        b'N' => Ok((i, Object::NULL)),
        b'T' => Ok((i, Object::Boolean(true))),
        b'F' => Ok((i, Object::Boolean(false))),
        0x80..=0xbf | 0xc0..=0xcf | 0xd0..=0xd7 | b'I' => {
            let (i, val) = read_int_bytag(i, tag)?;
            Ok((i, Object::Integer(val)))
        }
        0xd8..=0xef | 0xf0..=0xff | 0x38..=0x3f | 0x59 | b'L' => {
            let (i, val) = read_long_bytag(i, tag)?;
            Ok((i, Object::Long(val)))
        }
        0x5b..=0x5f | b'D' => {
            let (i, val) = read_double_bytag(i, tag)?;
            Ok((i, Object::Double(val)))
        }
        0x4a..=0x4b => {
            //Date mills
            let (i, val) = read_utcdate_bytag(i, tag)?;
            Ok((i, Object::Date(val as u64)))
        }
        0x00..=0x1f | 0x30..=0x33 | b'R' | b'S' => {
            let (i, val) = read_string_bytag(i, tag)?;
            Ok((i, Object::Str(val)))
        }
        0x20..=0x2f | 0x34..=0x37 | b'A' | b'B' => {
            let (i, val) = read_binary_bytag(i, tag)?;
            Ok((i, Object::Bin(val)))
        }
        0x55..=0x58 => {
            let (i, val) = read_list_bytag(i, tag)?;
            Ok((i, Object::List(val)))
        }
        _ => Ok((i, Object::NULL))
    };
    val
}

pub fn read_list_bytag(i: &[u8], tag: u8) -> Result<(&[u8], List), ParseErr> {
    let (i, val_type, len) = match tag {
        0x55 => {
            let (i, val_type) = read_type(i)?;
            (i, None, -1)
        }
        0x56 => {
            let (i, val_type) = read_type(i)?;
            let (i, len) = read_int(i)?;
            (i, Some(val_type), len)
        }
        0x58 => {
            let (i, len) = read_int(i)?;
            (i, None, len)
        }
        _ => (i, None, -1)
        // _ => Err(Err::Error(make_error(i, ErrorKind::Eof)))
    };

    let mut list = vec![];
    let mut offset = i;
    for _ in 0..len {
        let (i, obj) = read_object(offset)?;
        offset = i;
        list.push(obj);
    }

    match val_type {
        Some(val) => Ok((offset, List::Typed(val, list))),
        None => Ok((offset, List::UTyped(list)))
    }
}

pub fn read_list(input: &[u8]) -> Result<(&[u8], List), ParseErr> {
    let (i, tag) = be_u8(input)?;
    read_list_bytag(i, tag)
}

#[test]
pub fn test_read_binary() {
    let buf = read("d:/hessian.dat").unwrap();
    let mut ser = TypeRef::new(buf.as_ref());
    let vec = ser.read_binary().unwrap();
    println!("sef size {}", vec.len());
    let (i, byte_buff) = read_binary(buf.as_ref()).unwrap();
    let mut byte_count = 0;
    for idx in 0..byte_buff.len() {
        byte_count += 1;
    }

    let a = "";
    println!("chunk len {}, byte count is {}", byte_buff.len(), byte_count);


    let (i, str_val) = read_string(i).unwrap();
    println!("large text len {}", str_val.len());

    let (i, boolean) = read_bool(i).unwrap();
    let (i, int) = read_int(i).unwrap();
    let (i, long) = read_long(i).unwrap();
    let (i, double) = read_double(i).unwrap();
    println!("{:10.16e}", double);
    let (i, double) = read_double(i).unwrap();
    println!("{:10.1e}", double);

    let (i, utc) = read_utcdate(i).unwrap();
    let (i, str_val) = read_string(i).unwrap();
    println!("st {}", str_val);
    // let (i, val) = read_list(i).unwrap();
    let (i, obj) = read_object(i).unwrap();
    println!("{:?}", obj);
    let (i, obj) = read_object(i).unwrap();
    println!("{:?}", obj);
    let (i, obj) = read_object(i).unwrap();
    println!("{:?}", obj);
    let (i, obj) = read_object(i).unwrap();
    println!("{:?}", obj);
    let (i, obj) = read_object(i).unwrap();
    println!("{:?}", obj);
    let (i, obj) = read_object(i).unwrap();
    println!("date {:?}", obj);

    let (i, obj) = read_object(i).unwrap();
    println!("str object {:?}", obj);

    let (i, obj) = read_object(i).unwrap();
    match obj {
        Object::Bin(val) => println!("{}", std::str::from_utf8(val.as_slice()).unwrap()),
        _ => {}
    }

    let (i, list) = read_object(i).unwrap();
    println!("list object {:?}", list);

    let (i, list) = read_object(i).unwrap();
    println!("list object {:?}", list)
}
