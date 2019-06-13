use core::{mem, ptr, str};

use smallvec::SmallVec;

pub struct DataBlob {
    data: SmallVec<[u8; 0x8000]>,
    idx: usize,
}

impl DataBlob {
    pub fn new(data: SmallVec<[u8; 0x8000]>) -> DataBlob {
        DataBlob { data: data, idx: 0 }
    }

    pub fn get_idx(&self) -> usize {
        self.idx
    }

    pub fn skip(&mut self, skip_len: usize) {
        self.idx += skip_len
    }

    pub fn read_u8(&mut self) -> u8 {
        let number = self.data[self.idx];
        self.idx += 1;
        number
    }

    pub fn read_u16(&mut self) -> u16 {
        let bytes: &[u8] = &self.data[self.idx..self.idx + 2];

        let mut number: u16 = 0;
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), &mut number as *mut u16 as *mut u8, 2);
        };
        number.to_le();

        self.idx += 2;
        number
    }

    pub fn read_u32(&mut self) -> u32 {
        let bytes: &[u8] = &self.data[self.idx..self.idx + 4];

        let mut number: u32 = 0;
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), &mut number as *mut u32 as *mut u8, 4);
        };
        number.to_le();

        self.idx += 4;
        number
    }

    pub fn read_u64(&mut self) -> u64 {
        let bytes: &[u8] = &self.data[self.idx..self.idx + 8];

        let mut number: u64 = 0;
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), &mut number as *mut u64 as *mut u8, 8);
        };
        number.to_le();

        self.idx += 8;
        number
    }

    pub fn read_f32(&mut self) -> f32 {
        let number: f32 = unsafe { mem::transmute(self.read_u32()) };
        number
    }

    pub fn read_f64(&mut self) -> f64 {
        let number: f64 = unsafe { mem::transmute(self.read_u64()) };
        number
    }

    pub fn read_str(&mut self, str_len: usize) -> &str {
        if str_len == 0 {
            return "";
        }
        let text = str::from_utf8(&self.data[self.idx..self.idx + str_len]).unwrap();
        self.idx += str_len;
        text
    }

    pub fn read_u8_vec(&mut self, len: usize) -> SmallVec<[u8; 1024]> {
        let mut ret: SmallVec<[u8; 1024]> = SmallVec::new();

        ret.extend(self.data[self.idx..self.idx + len].iter().cloned());

        self.idx += len;
        ret
    }

    pub fn read_f32_vec(&mut self, len: usize) -> SmallVec<[f32; 1024]> {
        let mut ret: SmallVec<[f32; 1024]> = SmallVec::new();

        for _ in 0..len {
            ret.push(self.read_f32());
        }

        ret
    }
}

pub fn push_u32(v: &mut SmallVec<[u8; 64]>, n: u32) {
    let bytes = unsafe { mem::transmute::<_, [u8; 4]>(n.to_le()) };
    v.push(bytes[0]);
    v.push(bytes[1]);
    v.push(bytes[2]);
    v.push(bytes[3]);
}

pub fn push_f32(v: &mut SmallVec<[u8; 64]>, n: f32) {
    let val_u32: u32 = unsafe { mem::transmute(n) };
    push_u32(v, val_u32);
}

// NOTE: read_num_bytes and write_num_bytes macro in the byteorder crate by
// BurntSushi
//
// macro_rules! read_num_bytes {
//     ($ty:ty, $size:expr, $src:expr, $which:ident) => ({
//         assert!($size == ::core::mem::size_of::<$ty>());
//         assert!($size <= $src.len());
//         let mut data: $ty = 0;
//         unsafe {
//             copy_nonoverlapping(
//                 $src.as_ptr(),
//                 &mut data as *mut $ty as *mut u8,
//                 $size);
//         }
//         data.$which()
//     });
// }
//
// macro_rules! write_num_bytes {
//     ($ty:ty, $size:expr, $n:expr, $dst:expr, $which:ident) => ({
//         assert!($size <= $dst.len());
//         unsafe {
//             // N.B. https://github.com/rust-lang/rust/issues/22776
//             let bytes = transmute::<_, [u8; $size]>($n.$which());
//             copy_nonoverlapping((&bytes).as_ptr(), $dst.as_mut_ptr(), $size);
//         }
//     });
// }
