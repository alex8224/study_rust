use nom::number::complete::{be_u8, be_i32, be_i64, be_f64, be_u64, be_u32};
use nom::bytes::complete::take;
use nom::Err;

use std::fs::{read};
use nom::error::{ErrorKind, make_error};
use std::cell::{RefCell, Cell};
use std::fmt::Debug;
use std::collections::HashMap;

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
    Map,
}

macro_rules! as_obj_val {
    ($this: ident, $val_type: ident)  => {
        match $this {
            Object::$val_type(ref val) => Some(val),
            _ => None
        }
    }
}

impl Object {
    fn as_str(&self) -> Option<&String> {
        as_obj_val!(self, Str)
    }

    fn as_bool(&self) -> Option<&bool> {
        as_obj_val!(self, Boolean)
    }

    fn as_int(&self) -> Option<&i32> {
        as_obj_val!(self, Integer)
    }

    fn as_double(&self) -> Option<&f64> {
        as_obj_val!(self, Double)
    }

    fn as_long(&self) -> Option<&i64> {
        as_obj_val!(self, Long)
    }

    fn as_utc(&self) -> Option<&u64> {
        as_obj_val!(self, Date)
    }

    fn as_binary(&self) -> Option<&Vec<u8>> {
        as_obj_val!(self, Bin)
    }

    fn as_list(&self) -> Option<&Vec<Object>> {
        let val = as_obj_val!(self, List);
        match val {
            Some(list) => {
                match list {
                    List::UTyped(val) | List::Typed(_, val) => {
                        Some(val)
                    }
                    _ => None
                }
            }
            None => None
        }
    }
}

pub fn copy_slice(slice: &[u8], target: &mut Vec<u8>) {
    for idx in 0..slice.len() {
        target.push(slice[idx]);
    }
}

type ParseErr<'a> = Err<(&'a [u8], ErrorKind)>;

struct Serializer<'a> {
    type_ref: RefCell<Vec::<String>>,
    buff: RefCell<&'a [u8]>,
}

//class="Person;Person"
//#[derivde(Hessian, Debug, Display)]
#[derive(Debug)]
struct Person {
    name: String,
    age: i32,
    male: bool,
    attach: Vec<u8>,
    tel: Vec<String>,
}

//需要实现的效果，加入当前macro后，可以扫描当前对象的字段
impl Person {
    fn new() -> Person {
        Person {
            name: String::new(),
            age: 0,
            male: false,
            attach: vec![],
            tel: vec![],
        }
    }

    fn set(&mut self, val: HashMap<String, Object>) -> Result<(), ()> {
        val.into_iter().for_each(|(field, val)| {
            match field.as_str() {
                "name" => self.name = val.as_str().unwrap().to_string(),
                "age" => self.age = *val.as_int().unwrap(),
                "male" => self.male = *val.as_bool().unwrap(),
                "attach" => copy_slice(val.as_binary().unwrap(), &mut self.attach),
                "tel" => {
                    val.as_list().unwrap().into_iter().for_each(|f| {
                        self.tel.push(f.as_str().unwrap().to_string());
                    });
                }
                _ => {}
            }
        });

        Ok(())
    }
}

#[test]
fn create_person() {
    let mut p = Person::new();
    let mut pojo_map = HashMap::<String, Object>::new();
    pojo_map.insert("name".to_string(), Object::Str("张三丰".to_string()));
    pojo_map.insert("age".to_string(), Object::Integer(100));
    pojo_map.insert("male".to_string(), Object::Boolean(true));
    pojo_map.insert("attach".to_string(), Object::Bin(vec![1, 2, 3]));
    pojo_map.insert("tel".to_string(), Object::List(List::UTyped(vec![Object::Str("10086".to_string()), Object::Str("10010".to_string())])));
    p.set(pojo_map);
    println!("{:?}", &p);
}

impl<'a> Serializer<'a> {
    fn new(out_buff: &'a [u8]) -> Self {
        Self {
            type_ref: RefCell::new(Vec::<String>::new()),
            buff: RefCell::new(out_buff),
        }
    }

    fn get(&self, key: usize) -> String {
        let val_type = self.type_ref.borrow().get(key).unwrap().as_str().to_string();
        val_type
    }

    fn put(&self, val: String) -> usize {
        self.type_ref.borrow_mut().push(val);
        self.type_ref.borrow_mut().len() - 1
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

    fn read_bool(&self) -> Result<bool, ParseErr> {
        let (i, tag) = be_u8(*self.buff.borrow())?;
        self.buff.replace(i);
        if tag == 0x54 {
            //T
            Ok(true)
        } else {
            //F
            Ok(false)
        }
    }

    fn parse_utf8(&self) -> Result<u32, ParseErr> {
        let (i, u8_tag) = be_u8(*self.buff.borrow())?;
        let mut offset = i;
        let chr = if u8_tag < 0x80 {
            u8_tag as u32
        } else if (u8_tag & 0xe0) == 0xc0 {
            let (i, chr2) = be_u8(i)?;
            offset = i;
            let ch = (u32::from(u8_tag & 0x1f) << 6) + u32::from(chr2 & 0x3f);
            ch
        } else {
            let (i, chr2) = be_u8(i)?;
            let (i, chr3) = be_u8(i)?;
            offset = i;
            let ch = (u32::from(u8_tag & 0x0f) << 12) + (u32::from(chr2 & 0x3f) << 6) + u32::from(chr3 & 0x3f);
            ch
        };
        self.buff.replace(offset);
        Ok(chr)
    }

    fn merge_char(&self, len: usize) -> Result<String, ParseErr> {
        let mut str_val = String::with_capacity(len * 3);
        for _ in 0..len {
            let chr = self.parse_utf8()?;
            let chr = std::char::from_u32(chr).unwrap();
            str_val.push(chr);
        }
        Ok(str_val)
    }

    fn read_string_bytag(&self, tag: u8) -> Result<String, ParseErr> {
        let mut offset = *self.buff.borrow();
        let mut last_chunk = false;

        match tag {
            0x0..=0x1f => {
                let len = tag - 0x0;
                let str_val = self.merge_char(len as usize)?;
                Ok(str_val)
            }
            0x30..=0x33 => {
                let (i, tag2) = be_u8(offset)?;
                self.buff.replace(i);
                let len = u32::from(tag - 0x30) * 256 + u32::from(tag2);
                let str_val = self.merge_char(len as usize)?;
                offset = *self.buff.borrow();
                Ok(str_val)
            }
            0x52..=0x53 => {
                last_chunk = tag == 0x53;
                let mut str_buff = String::new();
                loop {
                    let (i, h) = be_u8(offset)?;
                    let (i, l) = be_u8(i)?;
                    // offset = i;
                    self.buff.replace(i);
                    let len = (u32::from(h) << 8) + u32::from(l);
                    let str_val = self.merge_char(len as usize)?;
                    offset = *self.buff.borrow();
                    str_buff.push_str(str_val.as_str());
                    if last_chunk {
                        break;
                    } else {
                        let (i, tag) = be_u8(offset)?;
                        last_chunk = tag == 0x53;
                        offset = i;
                    }
                }
                Ok(str_buff)
            }
            _ => Err(Err::Error(make_error(offset, ErrorKind::Eof)))
        }
    }

    fn read_string(&self) -> Result<String, ParseErr> {
        let (i, tag) = be_u8(*self.buff.borrow())?;
        self.buff.replace(i);
        self.read_string_bytag(tag)
    }


    fn read_int_bytag(&self, tag: u8) -> Result<i32, ParseErr> {
        match tag {
            0x80..=0xbf => {
                Ok((tag - 0x90) as i32)
            }
            0xc0..=0xcf => {
                //byte int
                let (i, l) = be_u8(self.cur_offset())?;
                let val = (i32::from(tag - 0xc8) << 8) + i32::from(l);
                self.incr_offset(i);
                Ok(val)
            }
            0xd0..=0xd7 => {
                //short
                let (i, h) = be_u8(self.cur_offset())?;
                let (i, l) = be_u8(i)?;
                let byte1 = (i32::from(tag) - 0xd4) << 16;
                let val = byte1 + 256 as i32 * i32::from(h) + i32::from(l);
                self.incr_offset(i);
                Ok(val)
            }
            0x49 | 0x59 => {
                //int
                let (i, val) = be_i32(self.cur_offset())?;
                Ok(val)
            }
            _ => Err(Err::Error(make_error(self.cur_offset(), ErrorKind::Eof)))
        }
    }

    fn incr_offset(&self, offset: &'a [u8]) {
        self.buff.replace(offset);
    }

    fn cur_offset(&self) -> &'a [u8] {
        &self.buff.borrow()
    }

    fn read_int(&self) -> Result<i32, ParseErr> {
        let (i, tag) = be_u8(self.cur_offset())?;
        self.incr_offset(i);
        self.read_int_bytag(tag)
    }


    fn read_long_bytag(&self, tag: u8) -> Result<i64, ParseErr> {
        match tag {
            0xd8..=0xef => {
                Ok(i64::from(tag - 0xe0))
            }
            0xf0..=0xff => {
                let (i, l) = be_u8(self.cur_offset())?;
                self.incr_offset(i);
                let val = ((i64::from(tag) - 0xf8) << 8) + i64::from(l);
                Ok(val)
            }
            0x38..=0x3f => {
                let (i, h) = be_u8(self.cur_offset())?;
                let (i, l) = be_u8(i)?;
                self.incr_offset(i);
                let val = ((i64::from(tag) - 0x3c) << 16) + 256 * i64::from(h) + i64::from(l);
                Ok(val)
            }
            0x49 | 0x59 => {
                let (i, val) = be_i32(self.cur_offset())?;
                self.incr_offset(i);
                Ok(val as i64)
            }
            0x4c => {
                let (i, val) = be_i64(self.cur_offset())?;
                self.incr_offset(i);
                Ok(val)
            }

            _ => Err(Err::Error(make_error(self.cur_offset(), ErrorKind::Eof)))
        }
    }

    fn read_long(&self) -> Result<i64, ParseErr> {
        let (i, tag) = be_u8(self.cur_offset())?;
        self.incr_offset(i);
        self.read_long_bytag(tag)
    }

    fn read_utcdate_bytag(&self, tag: u8) -> Result<u64, ParseErr> {
        match tag {
            0x4a => {
                // tag = J
                let (i, utc) = be_u64(self.cur_offset())?;
                self.incr_offset(i);
                Ok(utc)
            }
            0x4b => {
                // tag = K
                let (i, int) = be_u32(self.cur_offset())?;
                let utc = int as u64 * 60000;
                self.incr_offset(i);
                Ok(utc)
            }
            _ => Err(Err::Error(make_error(self.cur_offset(), ErrorKind::Eof)))
        }
    }

    fn read_utcdate(&self) -> Result<u64, ParseErr> {
        let (i, tag) = be_u8(self.cur_offset())?;
        self.incr_offset(i);
        self.read_utcdate_bytag(tag)
    }

    fn read_double_bytag(&self, tag: u8) -> Result<f64, ParseErr> {
        match tag {
            0x5b => {
                Ok(0 as f64)
            }
            0x5c => {
                Ok(1 as f64)
            }
            0x5d => {
                let (i, val) = be_u8(self.cur_offset())?;
                self.incr_offset(i);
                Ok(val as f64)
            }
            0x5e => {
                let (i, h) = be_u8(self.cur_offset())?;
                let (i, l) = be_u8(i)?;
                let val = u32::from(h) * 256 + u32::from(l);
                self.incr_offset(i);
                Ok(val as f64)
            }
            0x5f => {
                let (i, int) = be_i32(self.cur_offset())?;
                self.incr_offset(i);
                Ok(0.001 * int as f64)
            }
            0x44 => {
                // tag = D
                let (i, double) = be_f64(self.cur_offset())?;
                self.incr_offset(i);
                Ok(double)
            }
            _ => Err(Err::Error(make_error(self.cur_offset(), ErrorKind::Digit)))
        }
    }

    fn read_double(&self) -> Result<f64, ParseErr> {
        let (i, tag) = be_u8(self.cur_offset())?;
        self.incr_offset(i);
        self.read_double_bytag(tag)
    }

    fn read_type(&self) -> Result<String, ParseErr> {
        let (i, tag) = be_u8(self.cur_offset())?;
        match tag {
            //0x52 b'R' 0x53 b'S'
            0x0..=0x1f | 0x30..=0x33 | 0x52..=0x53 => {
                let val_type = self.read_string()?;
                let idx = self.put(val_type);
                Ok(self.get(idx))
            }
            _ => {
                if self.type_ref.borrow().is_empty() {
                    Err(Err::Error(make_error(self.cur_offset(), ErrorKind::Eof)))
                } else {
                    let type_ref = self.read_int()?;
                    Ok(self.get(type_ref as usize))
                }
            }
        }
    }

    fn read_object(&self) -> Result<Object, ParseErr> {
        let (i, tag) = be_u8(self.cur_offset())?;
        self.incr_offset(i);
        let val = match tag {
            b'N' => Ok(Object::NULL),
            b'T' => Ok(Object::Boolean(true)),
            b'F' => Ok(Object::Boolean(false)),
            0x80..=0xbf | 0xc0..=0xcf | 0xd0..=0xd7 | b'I' => {
                let val = self.read_int_bytag(tag)?;
                Ok(Object::Integer(val))
            }
            0xd8..=0xef | 0xf0..=0xff | 0x38..=0x3f | 0x59 | b'L' => {
                let val = self.read_long_bytag(tag)?;
                Ok(Object::Long(val))
            }
            0x5b..=0x5f | b'D' => {
                let val = self.read_double_bytag(tag)?;
                Ok(Object::Double(val))
            }
            0x4a..=0x4b => {
                //Date mills
                let val = self.read_utcdate_bytag(tag)?;
                Ok(Object::Date(val as u64))
            }
            0x00..=0x1f | 0x30..=0x33 | b'R' | b'S' => {
                let val = self.read_string_bytag(tag)?;
                Ok(Object::Str(val))
            }
            0x20..=0x2f | 0x34..=0x37 | b'A' | b'B' => {
                let val = self.read_binary_bytag(tag)?;
                Ok(Object::Bin(val))
            }
            0x55..=0x58 => {
                let val = self.read_list_bytag(tag)?;
                Ok(Object::List(val))
            }
            0x78..=0x7f => {
                //compact fixed untyped list
                let len = tag - 0x78;
                let mut rs = Vec::<Object>::new();
                for _ in 0..len {
                    rs.push(self.read_object()?);
                }
                Ok(Object::List(List::UTyped(rs)))
            }
            b'C' => {
                //readObjectDefinition(null);
                let obj_type = self.read_string()?;
                let len = self.read_int()?;
                //find serializer factory for special obj_type using java reflection
                //Object []fields = reader.createFields(len); //field serializer
                //String []fieldNames = new String[len]; //field names
                //add classref to cache
                for _ in 0..len {
                    let field = self.read_string()?;
                    println!("field {}", field);
                }

                // ObjectDefinition def = new ObjectDefinition(type, reader, fields, fieldNames);
                // readObject
                let obj = self.read_object()?;
                Ok(Object::NULL)
            }
            0x60..=0x6f => {
                //pojo
                let obj_ref = tag - 0x60;
                //get object definition from classref using ref
                //read Object Instance
                //find serializer for pojo type
                Ok((Object::NULL))
            }
            _ => Ok(Object::NULL)
        };
        val
    }

    fn read_list_bytag(&self, tag: u8) -> Result<List, ParseErr> {
        let (val_type, len) = match tag {
            0x55 => {
                let val_type = self.read_type()?;
                (None, -1)
            }
            0x56 => {
                let val_type = self.read_type()?;
                let len = self.read_int()?;
                (Some(val_type), len)
            }
            0x58 => {
                let len = self.read_int()?;
                (None, len)
            }
            _ => (None, -1)
        };

        let mut list = vec![];
        for _ in 0..len {
            let obj = self.read_object()?;
            list.push(obj);
        }

        match val_type {
            Some(val) => Ok(List::Typed(val.to_string(), list)),
            None => Ok(List::UTyped(list))
        }
    }

    fn read_list(&self) -> Result<List, ParseErr> {
        let (i, tag) = be_u8(self.cur_offset())?;
        self.incr_offset(i);
        self.read_list_bytag(tag)
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



#[test]
pub fn test_read_binary() {
    let buf = read("d:/hessian.dat").unwrap();
    let mut ser = Serializer::new(buf.as_ref());
    let vec = ser.read_binary().unwrap();
    let str = ser.read_string().unwrap();
    let boolean = ser.read_bool().unwrap();
    let int = ser.read_int().unwrap();
    let long = ser.read_long().unwrap();
    let double = ser.read_double().unwrap();
    let double1 = ser.read_double().unwrap();
    let utcdate = ser.read_utcdate().unwrap();
    let stval = ser.read_string().unwrap();
    println!("sef size {} string size {}, bool {} i32 is {}, i64 is {} double1 {:10.16e} double 2 {:10.1e}, \
    utcdate {} str {} ", vec.len(), str.len(), boolean, int, long, double, double1, utcdate, stval);
    for _ in 0..12 {
        let obj = ser.read_object();
        println!("{:?}", obj.unwrap());
    }
}
