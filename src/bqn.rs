use std::{ffi::c_char, ops::Deref, sync::Once};

use cbqn_sys::{
    bqn_bound, bqn_call1, bqn_call2, bqn_copy, bqn_eval, bqn_free, bqn_init, bqn_makeBoundFn1,
    bqn_makeBoundFn2, bqn_makeChar, bqn_makeF64, bqn_makeI8Vec, bqn_makeObjVec, bqn_makeUTF8Str,
    bqn_pick, bqn_rank, bqn_readC8Arr, bqn_readF64, bqn_readF64Arr, bqn_readI8Arr, bqn_shape,
};

#[allow(clippy::upper_case_acronyms)]
pub type BQNV = cbqn_sys::BQNV;

static mut TRAP: BQNV = 0;
static mut UTF8: BQNV = 0;
static INIT: Once = Once::new();

// poor man's https://github.com/Detegr/cbqn-rs

pub struct BQNValue(BQNV);

impl Deref for BQNValue {
    type Target = BQNV;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for BQNValue {
    fn drop(&mut self) {
        unsafe { bqn_free(self.0) }
    }
}

impl Clone for BQNValue {
    fn clone(&self) -> Self {
        BQNValue(unsafe { bqn_copy(self.0) })
    }
}

impl BQNValue {
    unsafe fn bqn_eval_str(src: &str) -> BQNV {
        let src = bqn_makeUTF8Str(src.len(), src.as_ptr() as *const c_char);
        bqn_eval(src)
    }

    pub fn copy(&self) -> BQNV {
        unsafe { bqn_copy(self.0) }
    }

    pub fn init() {
        INIT.call_once(|| {
            unsafe {
                bqn_init();
                TRAP = Self::bqn_eval_str("{1‿(𝕎 𝕩)}⎊{0‿(•CurrentError 𝕩)}");
                UTF8 = Self::bqn_eval_str("•ToUTF8");
            };
        })
    }

    pub fn eval(src: &str) -> Self {
        Self(unsafe { Self::bqn_eval_str(src) })
    }

    pub fn bound(&self) -> usize {
        unsafe { bqn_bound(self.0) }
    }
    pub fn shape(&self) -> Vec<usize> {
        unsafe {
            let rank = bqn_rank(self.0);
            let mut shape = Vec::with_capacity(rank);
            #[allow(clippy::uninit_vec)]
            shape.set_len(rank);
            bqn_shape(self.0, shape.as_mut_ptr());
            shape
        }
    }
    pub fn pick(&self, i: usize) -> Self {
        Self(unsafe { bqn_pick(self.0, i) })
    }

    pub fn null() -> Self {
        Self(unsafe { bqn_makeChar(0) })
    }

    pub fn to_f64(&self) -> f64 {
        unsafe { bqn_readF64(self.0) }
    }
    pub fn to_f64_vec(&self) -> Vec<f64> {
        unsafe {
            let bound = self.bound();
            let mut vec = Vec::with_capacity(bound);
            #[allow(clippy::uninit_vec)]
            vec.set_len(bound);
            bqn_readF64Arr(self.0, vec.as_mut_ptr());
            vec
        }
    }

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        unsafe {
            let utf8 = Self(bqn_call1(UTF8, self.0));
            let bound = utf8.bound();
            let mut bytes = Vec::with_capacity(bound);
            #[allow(clippy::uninit_vec)]
            bytes.set_len(bound);
            bqn_readC8Arr(utf8.0, bytes.as_mut_ptr());
            String::from_utf8(bytes).expect("Should be valid UTF8")
        }
    }
    pub fn to_string_vec(&self) -> Vec<String> {
        let bound = self.bound();
        let mut vec = Vec::with_capacity(bound);
        for i in 0..bound {
            let s = self.pick(i).to_string();
            vec.push(s);
        }
        vec
    }

    pub fn to_ptr<T>(&self) -> *mut T {
        unsafe {
            let bound = self.bound();
            let mut bytes = Vec::with_capacity(bound);
            #[allow(clippy::uninit_vec)]
            bytes.set_len(bound);
            bqn_readI8Arr(self.0, bytes.as_mut_ptr());
            let ptr = bytes.as_mut_ptr() as *const *mut T;
            *ptr
        }
    }

    pub fn call1(f: &BQNV, x: &BQNV) -> Self {
        Self(unsafe { bqn_call1(*f, *x) })
    }
    pub fn call2(f: &BQNV, w: &BQNV, x: &BQNV) -> Self {
        Self(unsafe { bqn_call2(*f, *w, *x) })
    }
    pub fn call_trap(f: &BQNV, x: &BQNV) -> Result<Self, String> {
        unsafe {
            let trap = TRAP;
            let r = Self::call2(&trap, f, x);
            let ok = r.pick(0).to_f64();
            if ok != 0.0 {
                let ret = r.pick(1);
                Ok(ret)
            } else {
                let err = r.pick(1).to_string();
                Err(err)
            }
        }
    }

    pub fn fn1(f: unsafe extern "C" fn(obj: BQNV, x: BQNV) -> BQNV, obj: &BQNV) -> Self {
        Self(unsafe { bqn_makeBoundFn1(Some(f), *obj) })
    }
    pub fn fn2(f: unsafe extern "C" fn(obj: BQNV, w: BQNV, x: BQNV) -> BQNV, obj: &BQNV) -> Self {
        Self(unsafe { bqn_makeBoundFn2(Some(f), *obj) })
    }
}

impl From<f64> for BQNValue {
    fn from(value: f64) -> Self {
        Self(unsafe { bqn_makeF64(value) })
    }
}

impl From<&str> for BQNValue {
    fn from(value: &str) -> Self {
        Self(unsafe { bqn_makeUTF8Str(value.len(), value.as_ptr() as *const c_char) })
    }
}
impl From<String> for BQNValue {
    fn from(value: String) -> Self {
        Self(unsafe { bqn_makeUTF8Str(value.len(), value.as_ptr() as *const c_char) })
    }
}

impl From<BQNV> for BQNValue {
    fn from(value: BQNV) -> Self {
        Self(value)
    }
}
impl From<&BQNV> for BQNValue {
    fn from(value: &BQNV) -> Self {
        Self(unsafe { bqn_copy(*value) })
    }
}

impl<const N: usize> From<[BQNValue; N]> for BQNValue {
    fn from(value: [BQNValue; N]) -> Self {
        let vec: Vec<BQNV> = value.into_iter().map(|v| v.copy()).collect();
        Self(unsafe { bqn_makeObjVec(vec.len(), vec.as_ptr()) })
    }
}

impl<T> From<*mut T> for BQNValue {
    fn from(value: *mut T) -> Self {
        let v = &value as *const *mut T;
        let v = v as *const i8;
        #[cfg(target_pointer_width = "64")]
        let n = 64 / 8;
        #[cfg(target_pointer_width = "32")]
        let n = 32 / 8;
        let mut vec = Vec::with_capacity(n);
        for i in 0..n {
            let byte = unsafe { *(v.add(i)) };
            vec.push(byte);
        }
        Self(unsafe { bqn_makeI8Vec(vec.len(), vec.as_ptr()) })
    }
}
