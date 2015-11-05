extern crate libc;
extern crate roaring;

use std::slice;
use std::io::Cursor;
use libc::{ c_char, c_uchar, c_void, size_t };
use roaring::RoaringBitmap;

#[no_mangle]
pub extern fn roaring_bitmap_create() -> *mut RoaringBitmap<u32> {
    Box::into_raw(Box::new(RoaringBitmap::new()))
}

#[no_mangle]
pub extern fn roaring_bitmap_create_with_capacity(
        _capacity: u32) -> *mut RoaringBitmap<u32> {
    roaring_bitmap_create()
}

#[no_mangle]
pub extern fn roaring_bitmap_of_ptr(
        len: size_t, data: *const u32) -> *mut RoaringBitmap<u32> {
    let bitmap = roaring_bitmap_create();
    roaring_bitmap_add_many(bitmap, len, data);
    bitmap
}

#[no_mangle]
pub extern fn roaring_bitmap_printf_describe(
        bitmap: *const RoaringBitmap<u32>) {
    println!("{:?}", unsafe { &*bitmap });
}

#[no_mangle]
pub extern fn roaring_bitmap_copy(
        bitmap: *const RoaringBitmap<u32>) -> *mut RoaringBitmap<u32> {
    Box::into_raw(Box::new(unsafe { &*bitmap }.clone()))
}

#[no_mangle]
pub extern fn roaring_bitmap_printf(
        bitmap: *const RoaringBitmap<u32>) {
    // What's the difference between this and roaring_bitmap_printf_describe?
    println!("{:?}", unsafe { &*bitmap });
}

#[no_mangle]
pub extern fn roaring_bitmap_and(
        left: *const RoaringBitmap<u32>,
        right: *const RoaringBitmap<u32>) -> *mut RoaringBitmap<u32> {
    Box::into_raw(Box::new(unsafe { &*left } & unsafe { &*right }))
}

#[no_mangle]
pub extern fn roaring_bitmap_and_inplace(
        left: *mut RoaringBitmap<u32>, right: *const RoaringBitmap<u32>) {
    unsafe { &mut *left }.intersect_with(unsafe { &*right });
}

#[no_mangle]
pub extern fn roaring_bitmap_or(
        left: *const RoaringBitmap<u32>,
        right: *const RoaringBitmap<u32>) -> *mut RoaringBitmap<u32> {
    Box::into_raw(Box::new(unsafe { &*left } | unsafe { &*right }))
}

#[no_mangle]
pub extern fn roaring_bitmap_or_inplace(
        left: *mut RoaringBitmap<u32>, right: *const RoaringBitmap<u32>) {
    unsafe { &mut *left }.union_with(unsafe { &*right });
}

#[no_mangle]
pub extern fn roaring_bitmap_or_many(
        len: size_t,
        bitmaps: *const *const RoaringBitmap<u32>) -> *mut RoaringBitmap<u32> {
    let mut result = RoaringBitmap::new();
    for &bitmap in unsafe { slice::from_raw_parts(bitmaps, len) } {
        result |= unsafe { &*bitmap };
    }
    Box::into_raw(Box::new(result))
}

#[no_mangle]
pub extern fn roaring_bitmap_or_many_heap(
        len: size_t,
        bitmaps: *const *const RoaringBitmap<u32>) -> *mut RoaringBitmap<u32> {
    let mut result = RoaringBitmap::new();
    for &bitmap in unsafe { slice::from_raw_parts(bitmaps, len) } {
        result |= unsafe { &*bitmap };
    }
    Box::into_raw(Box::new(result))
}

#[no_mangle]
pub extern fn roaring_bitmap_xor(
        left: *const RoaringBitmap<u32>,
        right: *const RoaringBitmap<u32>) -> *mut RoaringBitmap<u32> {
    Box::into_raw(Box::new(unsafe { &*left } ^ unsafe { &*right }))
}

#[no_mangle]
pub extern fn roaring_bitmap_xor_inplace(
        left: *mut RoaringBitmap<u32>, right: *const RoaringBitmap<u32>) {
    unsafe { &mut *left }.symmetric_difference_with(unsafe { &*right });
}

#[no_mangle]
pub extern fn roaring_bitmap_xor_many(
        len: size_t, bitmaps:
        *const *const RoaringBitmap<u32>) -> *mut RoaringBitmap<u32> {
    let mut result = RoaringBitmap::new();
    for &bitmap in unsafe { slice::from_raw_parts(bitmaps, len) } {
        result ^= unsafe { &*bitmap };
    }
    Box::into_raw(Box::new(result))
}

#[no_mangle]
pub extern fn roaring_bitmap_andnot(
        left: *const RoaringBitmap<u32>,
        right: *const RoaringBitmap<u32>) -> *mut RoaringBitmap<u32> {
    Box::into_raw(Box::new(unsafe { &*left } - unsafe { &*right }))
}

#[no_mangle]
pub extern fn roaring_bitmap_andnot_inplace(
        left: *mut RoaringBitmap<u32>, right: *const RoaringBitmap<u32>) {
    unsafe { &mut *left }.difference_with(unsafe { &*right });
}

#[no_mangle]
pub extern fn roaring_bitmap_free(bitmap: *mut RoaringBitmap<u32>) {
    unsafe { Box::from_raw(bitmap) };
}

#[no_mangle]
pub extern fn roaring_bitmap_add_many(
        bitmap: *mut RoaringBitmap<u32>, len: size_t, values: *const u32) {
    let values = unsafe { slice::from_raw_parts(values, len) };
    for &value in values {
        roaring_bitmap_add(bitmap, value);
    }
}

#[no_mangle]
pub extern fn roaring_bitmap_add(
        bitmap: *mut RoaringBitmap<u32>, value: u32) {
    unsafe { &mut *bitmap }.insert(value);
}

#[no_mangle]
pub extern fn roaring_bitmap_remove(
        bitmap: *mut RoaringBitmap<u32>, value: u32) {
    unsafe { &mut *bitmap }.remove(value);
}

#[no_mangle]
pub extern fn roaring_bitmap_contains(
        bitmap: *const RoaringBitmap<u32>, value: u32) -> bool {
    unsafe { &*bitmap }.contains(value)
}

#[no_mangle]
pub extern fn roaring_bitmap_get_cardinality(
        bitmap: *const RoaringBitmap<u32>) -> u64 {
    unsafe { &*bitmap }.len() as u64
}

#[no_mangle]
pub extern fn roaring_bitmap_is_empty(
        bitmap: *const RoaringBitmap<u32>) -> bool {
    unsafe { &*bitmap }.is_empty()
}

#[no_mangle]
pub extern fn roaring_bitmap_to_uint32_array(
        bitmap: *const RoaringBitmap<u32>,
        array: *mut u32) {
    let array = unsafe {
        slice::from_raw_parts_mut(array, usize::max_value())
    };
    for (i, value) in unsafe { &*bitmap }.iter().enumerate() {
        array[i] = value;
    }
}

#[no_mangle]
pub extern fn roaring_bitmap_remove_run_compression(
        _: *const RoaringBitmap<u32>) -> bool {
    false
}

#[no_mangle]
pub extern fn roaring_bitmap_run_optimize(
        _: *const RoaringBitmap<u32>) -> bool {
    false
}

#[no_mangle]
pub extern fn roaring_bitmap_shrink_to_fit(
        _: *const RoaringBitmap<u32>) -> size_t {
    0
}

#[no_mangle]
pub extern fn roaring_bitmap_portable_deserialize(
        buffer: *const c_char) -> *mut RoaringBitmap<u32> {
    let buffer = unsafe {
        slice::from_raw_parts(buffer as *const c_uchar, usize::max_value())
    };
    let bitmap = RoaringBitmap::deserialize_from(buffer).unwrap();
    Box::into_raw(Box::new(bitmap))
}

#[no_mangle]
pub extern fn roaring_bitmap_portable_size_in_bytes(
        bitmap: *const RoaringBitmap<u32>) -> size_t {
    let mut vec = Vec::new();
    unsafe { &*bitmap }.serialize_into(&mut vec).unwrap();
    vec.len()
}

#[no_mangle]
pub extern fn roaring_bitmap_portable_serialize(
        bitmap: *const RoaringBitmap<u32>,
        buffer: *mut c_char) -> size_t {
    let buffer = unsafe {
        slice::from_raw_parts_mut(buffer as *mut c_uchar, usize::max_value())
    };
    let mut cursor = Cursor::new(buffer);
    unsafe { &*bitmap }.serialize_into(&mut cursor).unwrap();
    cursor.position() as usize
}

#[no_mangle]
pub extern fn roaring_iterate(
        bitmap: *const RoaringBitmap<u32>,
        callback: extern fn(u32, *const c_void) -> bool,
        ptr: *const c_void) -> bool {
    unsafe { &*bitmap }.iter().all(|value| callback(value, ptr))
}

#[no_mangle]
pub extern fn roaring_bitmap_equals(
        left: *const RoaringBitmap<u32>,
        right: *const RoaringBitmap<u32>) -> bool {
    unsafe { &*left }.is_subset(unsafe { &*right })
    && unsafe { &*right }.is_subset(unsafe { &*left })
}

#[no_mangle]
pub extern fn roaring_bitmap_is_subset(
        left: *const RoaringBitmap<u32>,
        right: *const RoaringBitmap<u32>) -> bool {
    unsafe { &*left }.is_subset(unsafe { &*right })
}
