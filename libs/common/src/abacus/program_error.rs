use core::fmt::Debug;

pub enum ErrorCode {
    StackUnderflow,
    StackOverflow,
    InvalidInstruction,
    InvalidOperand,
    NotFound,
    OutOfRange,
    NotAligned,
    MathUnderflow,
    MathOverflow,
    SubroutineError(alloc::boxed::Box<ProgramError>),
}

pub struct ProgramError {
    pub error_code: ErrorCode,
    pub program_counter: usize,
    pub stack_depth: usize,
}

impl Debug for ErrorCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::StackUnderflow => write!(f, "StackUnderflow"),
            Self::StackOverflow => write!(f, "StackOverflow"),
            Self::InvalidInstruction => write!(f, "InvalidInstruction"),
            Self::InvalidOperand => write!(f, "InvalidOperand"),
            Self::NotFound => write!(f, "NotFound"),
            Self::OutOfRange => write!(f, "OutOfRange"),
            Self::NotAligned => write!(f, "NotAligned"),
            Self::MathUnderflow => write!(f, "MathUnderflow"),
            Self::MathOverflow => write!(f, "MathOverflow"),
            Self::SubroutineError(inner) => write!(f, "SubroutineError({:?})", *inner),
        }
    }
}

impl Debug for ProgramError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ProgramError")
            .field("error_code", &self.error_code)
            .field("program_counter", &self.program_counter)
            .field("stack_depth", &self.stack_depth)
            .finish()
    }
}
