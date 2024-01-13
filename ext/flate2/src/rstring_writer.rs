use magnus::{
    rb_sys::{protect, AsRawValue},
    RString,
};
use rb_sys::Qnil;

use crate::nogvl::with_gvl;

#[derive(Debug)]
pub struct RStringWriter {
    out: RString,
    offset: usize,
}

impl RStringWriter {
    pub fn new(out: RString) -> Self {
        Self {
            out,
            offset: out.len(),
        }
    }

    pub fn next_buffer(&mut self) -> Result<&mut [u8], &'static str> {
        let original_cap = self.out.capacity();
        let current_usable_size = original_cap.saturating_sub(self.offset);

        let outbuf_len = if current_usable_size == 0 {
            let new_capa = self.grow()?;
            new_capa.saturating_sub(self.offset)
        } else {
            current_usable_size
        };

        let slice = unsafe { self.out.as_slice() };
        let outbuf_ptr = unsafe { slice.as_ptr().add(self.offset) };

        Ok(unsafe { std::slice::from_raw_parts_mut(outbuf_ptr as *mut _, outbuf_len) })
    }

    pub fn finish(self) -> Result<RString, magnus::Error> {
        let new_len = self.offset;

        protect(|| unsafe {
            rb_sys::rb_str_set_len(self.out.as_raw(), new_len as _);
            Qnil as _
        })
        .expect("failed to resize flate2 output buffer");

        Ok(self.out)
    }

    pub fn consume(&mut self, amt: usize) -> Result<(), &'static str> {
        let new_len = self.offset + amt;

        // In order to avoid grabbing the GVL (which is very slow), we need to
        // ensure that the new length is less than the capacity.
        if new_len > self.out.capacity() {
            return Err("flate2 output buffer length is greater than capacity");
        }

        self.offset = new_len;

        Ok(())
    }

    fn grow(&mut self) -> Result<usize, &'static str> {
        let best_size = self.out.capacity() * 2;

        with_gvl(|| {
            protect(|| unsafe {
                rb_sys::rb_str_modify_expand(self.out.as_raw(), best_size as _);
                Qnil as _
            })
            .map_err(|_| "failed to increase flate2 output buffer capacity")
        })?;

        Ok(best_size)
    }
}
