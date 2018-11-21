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
}

impl From<u8> for OpCode {
    fn from(i: u8) -> OpCode {
        unsafe { std::mem::transmute(i) }
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
}
