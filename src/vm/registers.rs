const NUM_GP: usize = 16;
const NUM_FP: usize = 16;

#[derive(Debug, Clone)]
pub struct RegisterFile {
    pub gp: [i32; NUM_GP],
    pub fp: [f64; NUM_FP],
    pub pc: u32,
    pub sp: u32,
    pub flag_zero: bool,
    pub flag_sign: bool,
}

impl RegisterFile {
    pub fn new() -> Self {
        Self {
            gp: [0; NUM_GP],
            fp: [0.0; NUM_FP],
            pc: 0,
            sp: 0,
            flag_zero: false,
            flag_sign: false,
        }
    }

    #[inline]
    pub fn read_gp(&self, idx: u8) -> i32 {
        *self.gp.get(idx as usize).unwrap_or(&0)
    }

    #[inline]
    pub fn write_gp(&mut self, idx: u8, val: i32) {
        if (idx as usize) < NUM_GP {
            self.gp[idx as usize] = val;
        }
    }

    #[inline]
    pub fn read_fp(&self, idx: u8) -> f64 {
        *self.fp.get(idx as usize).unwrap_or(&0.0)
    }

    #[inline]
    pub fn write_fp(&mut self, idx: u8, val: f64) {
        if (idx as usize) < NUM_FP {
            self.fp[idx as usize] = val;
        }
    }

    #[inline]
    pub fn set_flags(&mut self, result: i32) {
        self.flag_zero = result == 0;
        self.flag_sign = result < 0;
    }
}

impl Default for RegisterFile {
    fn default() -> Self { Self::new() }
}
