#[repr(u8)]
#[derive(PartialEq, Debug)]
pub enum OpCode {
    NOOP,

    ADDA,
    ADDI,
    ADDR,

    ANDA,
    ANDI,
    ANDR,

    ORA,
    ORI,
    ORR,

    XORA,
    XORI,
    XORR,

    ROTR,
    ROTL,

    CALLA,
    CALLR,
    RTN,

    HALT,

    JMP,
    JMPA,

    LDSPA,
    LDSPI,
    LDSPR,

    LOADA,
    LOADI,
    LOADR,

    STRA,
    STRR,

    XFER,

    POP,
    PUSH,

    DEBUG,

    End, // Fake instruction marking end of enum list for conversion check
}

impl From<u8> for OpCode {
    fn from(i: u8) -> OpCode {
        if i > OpCode::End as u8 {
            OpCode::NOOP
        } else {
            unsafe { std::mem::transmute(i) }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_opcode_convert() {
        let i: u8 = 2;
        let op = OpCode::from(i);

        assert_eq!(op, OpCode::ADDI);
    }

    #[test]
    fn test_invalid_opcode_value() {
        let i: u8 = 255;
        let op = OpCode::from(i);

        assert_eq!(op, OpCode::NOOP);
    }
}
