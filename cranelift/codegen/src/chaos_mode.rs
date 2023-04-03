static mut EMPTY: &'static [u8] = &[];
static mut UNSTRUCTURED: &'static mut [u8] = EMPTY;

pub fn init_unstructured(data: &[u8]) {
    unsafe {
        UNSTRUCTURED = data;
    }
}

pub fn drop_unstructured() {
    unsafe {
        UNSTRUCTURED = EMPTY;
    }
}

pub fn get_mut<'a>() -> &'a mut [u8] {
    UNSTRUCTURED
}
