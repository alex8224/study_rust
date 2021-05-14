use nom::number::complete::{be_u8, be_u16, be_u32, be_u64};
use nom::bytes::complete::take;
use nom::error;
use nom::Err;

trait Decode<T> {
    fn decode(&self) -> T;
}

#[derive(Debug)]
pub struct Binary {
    data: Vec<u8>,
}

use std::io::prelude::*;
use std::fs::{read, write};
use nom::error::ErrorKind;
use nom::multi::many_till;
use std::cell::RefCell;
use crate::hessian::U8ByteCnt::{Single, Double, Three, Zero};

type ParseErr<'a> = Err<(&'a [u8], ErrorKind)>;

pub fn parse_byte_chunks(input: &[u8]) -> Result<(&[u8], &[u8]), ParseErr> {
    let (i, h) = be_u8(input)?;
    let (i, l) = be_u8(i)?;
    let len = (u32::from(h) << 8) + u32::from(l);
    let (i, val) = take(len as usize)(i)?;
    Ok((i, val))
}

pub fn read_binary<'a>(input: &'a [u8], byte_buff: &'a RefCell<Vec<&'a [u8]>>) -> Result<&'a [u8], ParseErr<'a>> {
    let (i, tag) = be_u8(input)?;
    let mut offset = i;
    let mut last_chunk = false;
     match tag {
        0x20..=0x2f => {
            //<=15
            let len = tag - 0x20;
            let (i, val) = take(len as usize)(i)?;
            Ok(i)
        }
        0x34..=0x37 => {
            // <=1023
            let (i, tag2) = be_u8(i)?;
            let len = u32::from(tag - 0x34) * 256 + u32::from(tag2);
            let (i, val) = take(len as usize)(i)?;
            offset = i;
            Ok(offset)
        }
        0x41..=0x42 => {
            last_chunk = tag == 0x42;
            loop {
                let (i, val) = parse_byte_chunks(offset)?;
                byte_buff.borrow_mut().push(val);
                offset = i;
                if last_chunk {
                    break;
                } else {
                    let (i, tag) = be_u8(offset)?;
                    offset = i;
                    last_chunk = tag == 0x42;
                }
            }
            Ok(offset)
        }
        _ => Ok(i)
    }
}

pub enum U8ByteCnt {
   Single, Double, Three, Zero
}

pub fn parse_u8<'a>(input: &'a [u8], byte_buff: &RefCell<Vec<u32>>) -> Result<(&'a [u8]), ParseErr<'a>> {

    let (i, u8_tag) = be_u8(input)?;
    let mut offset = i;
    return if u8_tag < 0x80 {
        byte_buff.borrow_mut().push(u8_tag as u32);
        Ok(i)
    } else if (u8_tag & 0xe0) == 0xc0 {
        let (i, chr2) = be_u8(i)?;
        offset = i;
        let ch = (u32::from(u8_tag & 0x1f) << 6) + u32::from(chr2 & 0x3f);
        byte_buff.borrow_mut().push(ch);
        Ok(offset)
    } else {
       // if (u8_tag& 0xf0) == 0xe0
        let (i, chr2) = be_u8(i)?;
        let (i, chr3) = be_u8(i)?;
        offset = i;
        let ch = (u32::from(u8_tag & 0x0f) << 12) + (u32::from(chr2 & 0x3f) << 6) + u32::from(chr3 & 0x3f);
        byte_buff.borrow_mut().push(ch);
        Ok(offset)
    }
}

pub fn merge_char<'a>(input: &'a [u8], byte_buff: &RefCell<Vec<u32>>, len: usize) -> Result<(&'a [u8]), ParseErr<'a>>{
    let mut offset = input;
    for idx in 0..len {
        let i = parse_u8(offset, &byte_buff)?;
        offset = i;
    }
    Ok(offset)
}

pub fn read_string(input: &[u8]) -> Result<(&[u8]), ParseErr> {
    let (i, tag) = be_u8(input)?;
    let mut offset = i;
    let mut last_chunk = false;

    let byte_buff =  RefCell::new(Vec::<u32>::new());
    let i = match tag {
        0x0..=0x1f => {
            let len = tag - 0x0;
            // (i, len as usize, Single)
            merge_char(i, &byte_buff, len as usize)?
        }
        0x30..=0x33 => {
            let (i, tag2) = be_u8(i)?;
            offset = i;
            let len = u32::from(tag - 0x30) * 256 + u32::from(tag2);
            // (i, len, Double)
            merge_char(offset, &byte_buff, len as usize)?
        }
        0x52..=0x53 => {
            last_chunk = tag == 0x53;
            loop {
                let (i, h) = be_u8(offset)?;
                let (i, l) = be_u8(i)?;
                offset = i;
                let len = (u32::from(h) << 8) + u32::from(l);
                offset = merge_char(offset, &byte_buff, len as usize)?;
                if last_chunk {
                    break;
                }else{
                    let (i, tag) = be_u8(offset)?;
                    last_chunk = tag == 0x53;
                    offset = i;
                }
            }
            offset
        }
        _ => i
    };
    // offset = i ;
    // offset = merge_char(i, &byte_buff, len as usize)?;
    let borrow_buff = byte_buff.borrow();
    let mut str_builder = String::new();
    //
    for e in 0..borrow_buff.len() {
        let chr = std::char::from_u32(*borrow_buff.get(e).unwrap()).unwrap();
        str_builder.push(chr);
    }
    println!("full text is {} {}", &str_builder, &str_builder.len());
    Ok(i)
}


#[test]
pub fn test_read_binary() {
    let buf = read("d:/hessian.dat").unwrap();
    let byte_buff = RefCell::new(Vec::<&[u8]>::new());
    let i = read_binary(buf.as_ref(), &byte_buff).unwrap();
    let i = read_string(i).unwrap();
    let mut sum_bytes = 0;
    let byte_arr = byte_buff.borrow();
    for i in 0..byte_arr.len() {
        let ele = byte_arr.get(i).unwrap();
        sum_bytes += ele.len();
    }

    println!("buff len {} ", sum_bytes);

    let mut slice_array = Vec::<u8>::new();
    for k in 0..byte_arr.len() {
        let ele = *byte_arr.get(k).unwrap();
        for r in ele {
            slice_array.push(*r);
        }
    }

    let chunked_text = std::str::from_utf8(slice_array.as_ref()).unwrap();
    // println!("{}", chunked_text);

    let c = '中' as u32;
    let b = std::char::from_u32(36825);
    println!("c {} {}", c, b.unwrap());
}
