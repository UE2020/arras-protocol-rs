#[derive(PartialEq)]
enum FasttalkType {
    Bool(bool),
    Number(f64),
    String(String)
}

impl FasttalkType {
    fn compare (blocka: Self, blockb: Self) -> bool {
        if std::mem::discriminant(&blocka) != std::mem::discriminant(&blockb) {
            false
        } else {
            match blocka {
                Self::Bool(a) => {
                    match blockb {
                        Self::Bool(b) => {
                            a == b
                        },
                        _ => panic!("Discriminant failed")
                    }
                }

                Self::Number(a) => {
                    match blockb {
                        Self::Number(b) => {
                            a == b
                        },
                        _ => panic!("Discriminant failed")
                    }
                }

                Self::String(a) => {
                    match blockb {
                        Self::String(b) => {
                            a == b
                        },
                        _ => panic!("Discriminant failed")
                    }
                }
            }
        }
    }

    fn is_number(&self) -> bool {
        match self {
            Self::Number(_) => true,
            _ => false
        }
    }

    fn as_number(&self) -> f64 {
        match self {
            Self::Number(v) => *v,
            _ => panic!("Invalid cast")
        }
    }

    fn is_string(&self) -> bool {
        match self {
            Self::String(_) => true,
            _ => false
        }
    }

    fn as_string(&self) -> String {
        match self {
            Self::String(v) => *v,
            _ => panic!("Invalid cast")
        }
    }
}

type Block = FasttalkType;

fn encode (message: Vec<Block>) -> Vec<u8> {
    let headers = vec![];
    let headerCodes = vec![];
    let mut contentSize = 0;
    let lastTypeCode = 0b1111;
    let repeatTypeCount = 0;
    for block in message {
        let mut typeCode = 0;
        if Block::compare(block, Block::Bool(false)) || Block::compare(block, Block::Number(0.)) {
            typeCode = 0b0000;
        } else if Block::compare(block, Block::Bool(true)) || Block::compare(block, Block::Number(1.)) {
            typeCode = 0b0001;
        } else if block.is_number() {
            let dec = block.as_number();
            if dec.fract() != 0. || dec < -0x100000000 as f64 || dec >= 0x100000000 as f64 {
                typeCode = 0b1000;
                contentSize += 4;
            } else if dec >= 0. {
                if dec < 0x100 as f64 {
                    typeCode = 0b0010;
                    contentSize += 1;
                } else if dec < 0x10000 as f64 {
                    typeCode = 0b0100;
                    contentSize += 2;
                } else if dec < 0x100000000 as f64 {
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
                } else if dec >= -0x100000000 as f64 {
                    typeCode = 0b0111;
                    contentSize += 4;
                }
            }
        }
    }
    Vec::new()
}
fn main() {
    println!("Hello, world!");
}
