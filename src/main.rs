#[derive(PartialEq, Clone)]
enum FasttalkType {
    Bool(bool),
    Number(f64),
    String(String),
}

impl FasttalkType {
    fn compare(blocka: &Self, blockb: &Self) -> bool {
        if std::mem::discriminant(blocka) != std::mem::discriminant(blockb) {
            false
        } else {
            match blocka {
                Self::Bool(a) => match blockb {
                    Self::Bool(b) => a == b,
                    _ => panic!("Discriminant failed"),
                },

                Self::Number(a) => match blockb {
                    Self::Number(b) => a == b,
                    _ => panic!("Discriminant failed"),
                },

                Self::String(a) => match blockb {
                    Self::String(b) => a == b,
                    _ => panic!("Discriminant failed"),
                },
            }
        }
    }

    fn is_number(&self) -> bool {
        match self {
            Self::Number(_) => true,
            _ => false,
        }
    }

    fn as_number(&self) -> f64 {
        match self {
            Self::Number(v) => *v,
            Self::Bool(v) => *v as u8 as f64,
            _ => panic!("Invalid cast"),
        }
    }

    fn is_string(&self) -> bool {
        match self {
            Self::String(_) => true,
            _ => false,
        }
    }

    fn as_string(&self) -> String {
        match self {
            Self::String(v) => v.clone(),
            _ => panic!("Invalid cast"),
        }
    }
}

type Block = FasttalkType;

fn encode(message: Vec<Block>) -> Vec<u8> {
    // SAFETY: These values will only be modified from 1 thread
    let mut u_32: [u8; 4] = [0; 4];
    let c_32: *mut [u8; 4] = &mut u_32 as *mut _;
    let f_32: *mut [u8; 4] = &mut u_32 as *mut _;

    let mut u_16: [u8; 2] = [0; 2];
    let c_16: *mut [u8; 2] = &mut u_16 as *mut _;

    let mut headers = vec![];
    let mut headerCodes = vec![];
    let mut contentSize = 0;
    let mut lastTypeCode = 0b1111;
    let mut repeatTypeCount = 0;
    for block in message.iter() {
        let mut typeCode = 0;
        if Block::compare(block, &Block::Bool(false)) || Block::compare(block, &Block::Number(0.)) {
            typeCode = 0b0000;
        } else if Block::compare(block, &Block::Bool(true))
            || Block::compare(block, &Block::Number(1.))
        {
            typeCode = 0b0001;
        } else if block.is_number() {
            let dec = block.as_number();
            if dec.fract() != 0.
                || dec < -0x100000000 as i64 as f64
                || dec >= 0x100000000 as i64 as f64
            {
                typeCode = 0b1000;
                contentSize += 4;
            } else if dec >= 0. {
                if dec < 0x100 as f64 {
                    typeCode = 0b0010;
                    contentSize += 1;
                } else if dec < 0x10000 as f64 {
                    typeCode = 0b0100;
                    contentSize += 2;
                } else if dec < 0x100000000 as i64 as f64 {
                    typeCode = 0b0110;
                    contentSize += 4;
                }
            } else {
                if dec >= -0x100 as f64 {
                    typeCode = 0b0011;
                    contentSize += 1;
                } else if dec >= -0x10000 as f64 {
                    typeCode = 0b0101;
                    contentSize += 2;
                } else if dec >= -0x100000000 as i64 as f64 {
                    typeCode = 0b0111;
                    contentSize += 4;
                }
            }
        } else if block.is_string() {
            let dec = block.as_string();
            let mut hasUnicode = !dec.chars().all(|c| char::is_ascii(&c));
            println!("hasUnicode {}", hasUnicode);
            if !hasUnicode && dec.chars().collect::<Vec<_>>().len() <= 1 {
                typeCode = 0b1001;
                contentSize += 1;
            } else if hasUnicode {
                typeCode = 0b1011;
                contentSize += dec.chars().collect::<Vec<_>>().len() * 4;
            } else {
                typeCode = 0b1010;
                contentSize += dec.chars().collect::<Vec<_>>().len() + 1;
            }
        } else {
            panic!("Memory corruption");
        }
        headers.push(typeCode);
        if typeCode == lastTypeCode {
            repeatTypeCount += 1;
        } else {
            headerCodes.push(lastTypeCode);
            if repeatTypeCount >= 1 {
                while repeatTypeCount > 19 {
                    headerCodes.push(0b1110);
                    headerCodes.push(15);
                    repeatTypeCount -= 19;
                }
                if repeatTypeCount == 1 {
                    headerCodes.push(0b1100);
                } else if repeatTypeCount == 2 {
                    headerCodes.push(0b1100);
                } else if repeatTypeCount == 3 {
                    headerCodes.push(0b1101);
                } else if repeatTypeCount < 20 {
                    headerCodes.push(0b1110);
                    headerCodes.push(repeatTypeCount - 4);
                }
            }
            repeatTypeCount = 0;
            lastTypeCode = typeCode;
        }
    }
    headerCodes.push(lastTypeCode);
    if repeatTypeCount >= 1 {
        while repeatTypeCount > 19 {
            headerCodes.push(0b1110);
            headerCodes.push(15);
            repeatTypeCount -= 19;
        }
        if repeatTypeCount == 1 {
            headerCodes.push(lastTypeCode);
        } else if repeatTypeCount == 2 {
            headerCodes.push(0b1100);
        } else if repeatTypeCount == 3 {
            headerCodes.push(0b1101);
        } else if repeatTypeCount < 20 {
            headerCodes.push(0b1110);
            headerCodes.push(repeatTypeCount - 4);
        }
    }
    headerCodes.push(0b1111);
    if headerCodes.len() % 2 == 1 {
        headerCodes.push(0b1111);
    }
    let mut output = Vec::with_capacity((headerCodes.len() >> 1) + contentSize);
    println!("Output {}", (headerCodes.len() >> 1) + contentSize);
    println!("Content size {}", contentSize);
    println!("Header codes {}", headerCodes.len());
    println!("opcode {}", headerCodes.len() >> 1);

    output.resize((headerCodes.len() >> 1) + contentSize, 0);
    // loop
    let mut i = 0;
    while i < headerCodes.len() {
        let upper = headerCodes[i];
        let lower = headerCodes[i + 1];
        output[i >> 1] = (upper << 4) | lower;
        i += 2;
    }
    i = 0;

    let mut index = headerCodes.len() >> 1;
    for i in 0..headers.len() {
        let block = message[i].clone();
        loop {
            match headers[i] {
                0b0000 | 0b0001 => break,
                0b0010 | 0b0011 => {
                    let idx = index;
                    index += 1;
                    match block {
                        Block::Bool(v) => output[idx] = v as u8,
                        Block::Number(v) => output[idx] = v as u8,
                        _ => (),
                    }
                    break;
                }
                0b0100 | 0b0101 => {
                    u_16 = unsafe { std::mem::transmute::<u16, [u8; 2]>(block.as_number() as u16) };
                    let mut j = 0;
                    let offset = index;
                    for value in &unsafe { *c_16 } {
                        output[j + offset] = *value;
                        j += 1;
                    }
                    index += 2;
                    break;
                }
                0b0110 | 0b0111 => {
                    u_32 = unsafe { std::mem::transmute::<u32, [u8; 4]>(block.as_number() as u32) };
                    let mut j = 0;
                    let offset = index;
                    for value in &unsafe { *c_32 } {
                        output[j + offset] = *value;
                        j += 1;
                    }
                    index += 4;
                    break;
                }
                0b1000 => {
                    u_32 = unsafe { std::mem::transmute::<f32, [u8; 4]>(block.as_number() as f32) };
                    let mut j = 0;
                    let offset = index;
                    for value in &unsafe { *c_32 } {
                        output[j + offset] = *value;
                        j += 1;
                    }
                    index += 4;
                    break;
                }
                0b1001 => {
                    let block = block.as_string();
                    let byte = if block.chars().collect::<Vec<_>>().len() == 0 {
                        0
                    } else {
                        block.chars().nth(i).unwrap() as u32
                    };
                    let idx = index;
                    index += 1;
                    output[idx] = byte as u8;
                    break;
                }
                0b1010 => {
                    let block = block.as_string();
                    for chara in block.chars() {
                        let idx = index;
                        index += 1;
                        output[idx] = chara as u8;
                    }
                    let idx = index;
                    index += 1;
                    output[idx] = 0;

                }
                0b1011 => {
                    let block = block.as_string();
                    for chara in block.encode_utf16() {
                        let charCode = chara;
                        println!("{}", charCode);
                        let idx = index;
                        index += 1;
                        println!("{}", (charCode & 0xff));
                        output[idx] = (charCode & 0xff) as u8;

                        let idx = index;
                        println!("{}", (charCode >> 8));
                        index += 1;
                        output[idx] = (charCode >> 8) as u8;
                    }
                    let idx = index;
                    index += 1;
                    output[idx] = 0;

                    let idx = index;
                    index += 1;
                    output[idx] = 0;
                }
                _ => break
            }
            break;
        }
    }
    output
}
fn main() {
    use std::time::{Duration, Instant};
    // lets bench it
    let start = Instant::now();
    for i in 0..1000 {
        let payload = vec![
            Block::Bool(true),
            Block::Bool(true),
            Block::Bool(true),
            Block::Bool(true),
            Block::String("haha arras ezz".to_owned()),
            Block::Number(0.4),
            Block::Number(3.14),
            Block::Number(1.8),
        ];
        encode(payload);
    }
    let duration = start.elapsed();
    println!("arras_protocol: 1000 cycles took {:?}", duration);

    // some tests
    let payload = vec![
        Block::Bool(true),
        Block::Bool(true),
        Block::Bool(true),
        Block::Bool(true),
        Block::String("𝓱𝓪𝓱𝓪 𝓮𝔃𝔃".to_owned()),
        Block::Number(0.4),
        Block::Number(3.14),
        Block::Number(1.8),
    ];
    println!("arras_protocol-test1: {:?}", encode(payload));
}
