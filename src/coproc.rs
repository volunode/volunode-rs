pub const MAX_RSC: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProcType {
    CPU,
    NVIDIAGraphics,
    AMDGraphics,
    IntelGraphics,
    MinerASIC,
}

#[derive(Clone, Debug, Default)]
pub struct Coprocs {}
