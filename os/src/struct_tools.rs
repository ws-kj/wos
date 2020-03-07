use core::mem;
use core::slice;

pub unsafe fn to_slice<T: Sized>(p: &T) -> &[u8] {
    slice::from_raw_parts(
        (p as *const T) as *const u8,
        mem::size_of::<T>(),
     )
}
