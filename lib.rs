extern crate quick_xml;
use std::io::Read;
use std::time::SystemTime;
use std::{
    borrow::{Borrow, BorrowMut},
    env,
};
use std::{fs::File, str::FromStr};

use quick_xml::events::Event;
use quick_xml::Reader;

// This is the interface to the JVM that we'll
// call the majority of our methods on.
use jni::JNIEnv;

// These objects are what you should use as arguments to your native function.
// They carry extra lifetime information to prevent them escaping this context
// and getting used after being GC'd.
use jni::objects::{GlobalRef, JClass, JObject, JString, JValue};

// This is just a pointer. We'll be returning it from our function.
// We can't return one of the objects with lifetime information because the
// lifetime checker won't let us.
use jni::sys::{jbyteArray, jclass, jint, jlong, jmethodID, jobject, jstring};

use std::{sync::mpsc, thread, time::Duration};

struct Message<'a> {
    msg_type: Option<JString<'a>>,
    msg_id: Option<JString<'a>>,
}

impl<'a> Message<'a> {
    fn new() -> Self {
        Message {
            msg_type: None,
            msg_id: None,
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_HelloWorld_parseXml(
    env: JNIEnv,
    _self: JObject,
    msg_obj: JObject,
    xml: JString,
) {
    let buff: String = env
        .get_string(xml)
        .expect("convert to native str failed!")
        .into();
    let mut byte_arr = Vec::new();
    let mut node = String::from("");
    let startTime = SystemTime::now();
    for i in 0..10000 {
        let mut reader = Reader::from_str(buff.as_str());

        let mut msg = Message::new();
        loop {
            let evt = reader.read_event(&mut byte_arr).unwrap();
            match evt {
                Event::Start(ref e) => {
                    node = std::str::from_utf8(e.name()).unwrap().to_string();
                    // println!("start tag {}", node);
                }
                Event::End(ref e) => {
                    // println!("end tag {}", node)
                }
                Event::Empty(ref e) => {
                    // println!("empty tag {}", std::str::from_utf8(e.name()).unwrap())
                }
                Event::Text(ref e) => {
                    println!("tag text node {}", std::str::from_utf8(e.escaped()).unwrap());
                    let val = env
                        .new_string(std::str::from_utf8(e.escaped()).unwrap().to_string())
                        .unwrap();
                    match node.as_str() {
                        "msgType" => {
                            if msg.msg_type.is_none() {
                                msg.msg_type = Option::Some(val);
                            }
                        }
                        "msgId" => {
                            if msg.msg_id.is_none() {
                                msg.msg_id = Option::Some(val);
                            }
                        }
                        "string" => {
                            // byte_arr.clear();
                            // node.clear();
                            // println!(
                            //     "msgId is {:?}, msgType is {:?} body is {:?}",
                            //     msg.msgId, msg.msgType, msg.full_body
                            // );
                            break;
                        }
                        _ => (),
                    }
                }
                Event::Decl(ref e) => {}

                Event::CData(ref cdata) => {}
                Event::Eof | _ => {
                    byte_arr.clear();
                    break;
                }
            };
        }
    }

    // env.set_field(
    //     msg_obj,
    //     "msgType",
    //     "Ljava/lang/String;",
    //     msg.msg_type.unwrap().into(),
    // )
    // .expect("set msgType failed!");
    // env.set_field(
    //     msg_obj,
    //     "msgId",
    //     "Ljava/lang/String;",
    //     msg.msg_id.unwrap().into(),
    // )
    // .expect("set msgType failed!");
    println!("耗时 {}", startTime.elapsed().unwrap().as_millis());
}
