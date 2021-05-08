extern crate quick_xml;
extern crate serde;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::Reader;
use quick_xml::Writer;
use std::io::{Cursor, Read};
use std::iter;

#[test]
fn test_basic_xml() {
    println!("oj");

    let xml = r#"<tag1 att1 = "test">
                <tag2><!--Test comment-->Test</tag2>
                <tag2>
                    Test 2
                </tag2>
            </tag1>"#;

    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut count = 0;
    let mut txt = Vec::new();
    let mut buf = Vec::new();

    // The `Reader` does not implement `Iterator` because it outputs borrowed data (`Cow`s)
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"tag1" => println!(
                    "attributes values: {:?}",
                    e.attributes().map(|a| a.unwrap().value).collect::<Vec<_>>()
                ),
                b"tag2" => count += 1,
                _ => (),
            },
            Ok(Event::Text(e)) => txt.push(e.unescape_and_decode(&reader).unwrap()),
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (), // There are several other `Event`s we do not consider here
        }

        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }
}

#[test]
fn test_xml_write() {
    let xml = r#"<this_tag k1="v1" k2="v2"><child>text</child></this_tag>"#;
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) if e.name() == b"this_tag" => {
                // crates a new element ... alternatively we could reuse `e` by calling
                // `e.into_owned()`
                let mut elem = BytesStart::owned(b"my_elem".to_vec(), "my_elem".len());

                // collect existing attributes
                elem.extend_attributes(e.attributes().map(|attr| attr.unwrap()));

                // copy existing attributes, adds a new my-key="some value" attribute
                elem.push_attribute(("my-key", "some value"));

                // writes the event to the writer
                assert!(writer.write_event(Event::Start(elem)).is_ok());
            }
            Ok(Event::End(ref e)) if e.name() == b"this_tag" => {
                assert!(writer
                    .write_event(Event::End(BytesEnd::borrowed(b"my_elem")))
                    .is_ok());
            }
            Ok(Event::Eof) => break,
            // you can use either `e` or `&e` if you don't want to move the event
            Ok(e) => assert!(writer.write_event(&e).is_ok()),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
        buf.clear();
    }

    let result = writer.into_inner().into_inner();
    let expected = r#"<my_elem k1="v1" k2="v2" my-key="some value"><child>text</child></my_elem>"#;
    assert_eq!(result, expected.as_bytes());
}

use minidom::Element;
use minidom::Error;
use minidom::Result;
use std::fs::File;

fn split_element_name<S: AsRef<str>>(s: S) -> Result<(Option<String>, String)> {
    let name_parts = s.as_ref().split(':').collect::<Vec<&str>>();
    match name_parts.len() {
        2 => Ok((Some(name_parts[0].to_owned()), name_parts[1].to_owned())),
        1 => Ok((None, name_parts[0].to_owned())),
        _ => Err(Error::InvalidElement),
    }
}

use std::time::SystemTime;
// #[test]
pub fn test_dom(cnt: u32) {
    let mut file = File::open("c://users//administrator//desktop//1.xml").unwrap();
    let mut buff = String::new();
    let size = file.read_to_string(&mut buff).unwrap();
    // println!("read {} size content {}", size, buff);
    
    let start = SystemTime::now();
    let mut byteArr = Vec::new();
    let mut reader = Reader::from_str(buff.as_str());
    for i in 1..cnt {
        reader.read_event(&mut byteArr).unwrap();
    }
    

    println!("耗时{}", start.elapsed().unwrap().as_millis());
    // let mut reader = Reader::from_str(buff.as_str());
    // let mut byteArr = Vec::new();
    // loop {
    //     let e = reader.read_event(&mut byteArr).unwrap();
    //     match e {
    //         Event::Empty(ref e) | Event::Start(ref e) => {
    //             let evt_name = std::str::from_utf8(e.name()).unwrap();
    //             let (ns, ndname)  = split_element_name(evt_name).unwrap();
    //             println!("{} \n{:?}, {}", evt_name, ns, ndname);
    //             break;
    //         }
    //         Event::Eof => {
    //             break;
    //         }
    //         Event::Comment { .. } => {
    //             break;
    //         }
    //         Event::Text { .. }
    //         | Event::End { .. }
    //         | Event::CData { .. }
    //         | Event::Decl { .. }
    //         | Event::PI { .. }
    //         | Event::DocType { .. } => (), // TODO: may need more errors
    //     }
    // }
}

#[test]
fn test_pure_event_xml() {}
