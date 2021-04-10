use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::*;
mod fasttalk;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

#[wasm_bindgen]
pub fn encode(input: Vec<JsValue>) -> Vec<u8> {
    let mut packet: Vec<fasttalk::Block> = Vec::new();
    for value in input {
        if value.is_string() {
            packet.push(fasttalk::Block::String(value.as_string().unwrap()));
        } else {
            match value.as_bool() {
                Some(b) => packet.push(fasttalk::Block::Bool(b)),
                None => {
                    match value.as_f64() {
                        Some(i) => packet.push(fasttalk::Block::Number(i)),
                        None => ()
                    }
                }
            }
        }
    }
    console::log_1(&JsValue::from(&format!("Packet {:?}", packet)));
    fasttalk::encode(packet)
    
}