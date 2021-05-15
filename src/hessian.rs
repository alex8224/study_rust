use nom::number::complete::{be_u8, be_i32, be_i64, be_f64, be_u64, be_u32};
use nom::bytes::complete::take;
use nom::Err;

trait Decode<T> {
    fn decode(&self) -> T;
}

#[derive(Debug)]
pub struct Binary {
    data: Vec<u8>,
}

use std::fs::{read, write};
use nom::error::{ErrorKind, make_error};
use std::cell::RefCell;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc, NaiveDateTime, NaiveTime, Local, TimeZone};

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
            byte_buff.borrow_mut().push(val);
            Ok(i)
        }
        0x34..=0x37 => {
            // <=1023
            let (i, tag2) = be_u8(i)?;
            let len = u32::from(tag - 0x34) * 256 + u32::from(tag2);
            let (i, val) = take(len as usize)(i)?;
            byte_buff.borrow_mut().push(val);
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

pub fn parse_u8<'a>(input: &'a [u8], byte_buff: &RefCell<Vec<u32>>) -> Result<&'a [u8], ParseErr<'a>> {

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

pub fn merge_char<'a>(input: &'a [u8], byte_buff: &RefCell<Vec<u32>>, len: usize) -> Result<&'a [u8], ParseErr<'a>>{
    let mut offset = input;
    for _ in 0..len {
        let i = parse_u8(offset, &byte_buff)?;
        offset = i;
    }
    Ok(offset)
}

pub fn read_string(input: &[u8]) -> Result<&[u8], ParseErr> {
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
    // println!("full text is {} {}", &str_builder, &str_builder.len());
    Ok(i)
}

pub fn read_bool(input: &[u8]) -> Result<(&[u8], bool), ParseErr> {
    let (i, tag) = be_u8(input)?;
    if tag == 0x54 {
        Ok((i, true))
    }else{
        Ok((i, false))
    }
}

pub fn read_int(input: &[u8]) -> Result<(&[u8], i32), ParseErr> {
    let (i, tag) = be_u8(input)?;
    match tag {
        0xcc..=0xcf => {
            //byte int
            let (i, l) = be_u8(i)?;
            let val = (i32::from(tag - 0xc8) << 8) + i32::from(l);
            Ok((i, val))
        },
        0xd0..=0xd7 => {
            //short
            let (i, h) = be_u8(i)?;
            let (i, l) = be_u8(i)?;
            let byte1 =  (i32::from(tag) - 0xd4 ) << 16;
            let val = byte1 + 256 as i32 * i32::from(h)  + i32::from(l);
            Ok((i, val))
        },
        0x49 | 0x59 => {
            //int
            let (i, val) = be_i32(i)?;
            Ok((i, val))
        }
        _ => Err(Err::Error(make_error(input, ErrorKind::Eof)))
    }
}

pub fn read_long(input: &[u8]) -> Result<(&[u8], i64), ParseErr> {
    let (i, tag) = be_u8(input)?;
    match tag {
        0xd8..=0xef => {
            Ok((i, i64::from(tag - 0xe0)))
        },
        0xf0..=0xff => {
            let (i, l) = be_u8(i)?;
            let val = ((i64::from(tag) - 0xf8) << 8) + i64::from(l);
            Ok((i, val))
        },
        0x38..=0x3f => {
            let (i, h) = be_u8(i)?;
            let (i, l) = be_u8(i)?;
            let val = ((i64::from(tag) - 0x3c) << 16) + 256 * i64::from(h) + i64::from(l);
            Ok((i, val))
        },
        0x49 | 0x59 => {
            let (i, val) = be_i32(i)?;
            Ok((i, val as i64))
        }
        0x4c => {
            let (i, val) = be_i64(i)?;
            Ok((i, val))
        }

        _ =>  Err(Err::Error(make_error(input, ErrorKind::Eof)))
    }
}

fn read_utcdate(input: &[u8])-> Result<(&[u8], u64), ParseErr> {
    let (i, tag) = be_u8(input)?;
    match tag {
        0x4a => {
            // tag = J
            let (i, utc) = be_u64(i)?;
            Ok((i, utc))
        },
        0x4b => {
            // tag = K
            let (i, int) = be_u32(i)?;
            let utc = int as u64 * 60000;
            Ok((i, utc))
        }
        _ =>  Err(Err::Error(make_error(input, ErrorKind::Eof)))
    }
}

pub fn read_double(input: &[u8]) -> Result<(&[u8], f64), ParseErr> {
    let (i, tag) = be_u8(input)?;
    match tag {
        0x5b => {
            Ok((i, 0 as f64))
        },
        0x5c => {
            Ok((i, 1 as f64))
        },
        0x5d => {
            let (i, val) = be_u8(i)?;
            Ok((i, val as f64))
        },
        0x5e => {
            let (i, h) = be_u8(i)?;
            let (i, l) = be_u8(i)?;
            let val = u32::from(h) * 256 + u32::from(l);
            Ok((i, val as f64))
        },
        0x5f => {
            let (i, int) = be_i32(i)?;
            Ok((i, 0.001 * int as f64))
        },
        0x44 => {
            // tag = D
            let (i, double) = be_f64(i)?;
            Ok((i, double))
        },
        _ =>  Err(Err::Error(make_error(input, ErrorKind::Eof)))
    }
}


#[test]
pub fn test_read_binary() {
    let buf = read("d:/hessian.dat").unwrap();
    let byte_buff = RefCell::new(Vec::<&[u8]>::new());
    let i = read_binary(buf.as_ref(), &byte_buff).unwrap();
    let i = read_string(i).unwrap();
    let (i, boolean) = read_bool(i).unwrap();
    let (i, int) = read_int(i).unwrap();
    let (i, long) = read_long(i).unwrap();
    let (i, double) = read_double(i).unwrap();
    println!("{:10.16e}", double);
    let (i, double) = read_double(i).unwrap();
    println!("{:10.1e}", double);
    let(i, utc) = read_utcdate(i).unwrap();
    let d = Duration::from_millis(utc);
    let natime = NaiveDateTime::from_timestamp(d.as_secs() as i64, 0);
    // let date = DateTime::<Utc>::from_utc(natime);
    let date_time: DateTime<Local> = Local.from_local_datetime(&natime).unwrap();
    println!("{:?}", date_time);
    println!("{:}", natime);

    let mut sum_bytes = 0;
    let byte_arr = byte_buff.borrow();
    for i in 0..byte_arr.len() {
        let ele = byte_arr.get(i).unwrap();
        sum_bytes += ele.len();
    }

    println!("buff len {} ", sum_bytes);

    // let mut slice_array = Vec::<u8>::with_capacity(byte_arr.len());
    // for k in 0..byte_arr.len() {
    //     let ele = *byte_arr.get(k).unwrap();
    //     for r in ele {
    //         slice_array.push(*r);
    //     }
    // }
    //
    // let chunked_text = std::str::from_utf8(slice_array.as_ref()).unwrap();
    // println!("{}", chunked_text);
}
