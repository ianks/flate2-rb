use std::{ffi::c_void, mem::MaybeUninit, ptr::null_mut};

use rb_sys::{rb_thread_call_with_gvl, rb_thread_call_without_gvl};

#[derive(Debug, thiserror::Error)]
pub enum InterruptableError {
    #[error("interrupted")]
    Interrupt,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("internal error: {0}")]
    Internal(&'static str),
}

pub fn nogvl_with_interrupt_callback<F, R, I>(mut func: F, mut interrupt: I) -> R
where
    F: FnMut() -> R,
    R: Sized,
    I: FnMut(),
{
    let result = MaybeUninit::uninit();
    let arg_ptr = &(&mut func, &result) as *const _ as *mut c_void;
    let interrupt_ptr = &(&mut interrupt) as *const _ as *mut c_void;

    unsafe {
        rb_thread_call_without_gvl(
            Some(ffi_wrap::<F, R>),
            arg_ptr,
            Some(ffi_wrap_interrupt::<I>),
            interrupt_ptr,
        );
        result.assume_init()
    }
}

pub fn with_gvl<F, R>(mut func: F) -> R
where
    F: FnMut() -> R,
    R: Sized,
{
    let result = MaybeUninit::uninit();
    let arg_ptr = &(&mut func, &result) as *const _ as *mut c_void;

    unsafe {
        rb_thread_call_with_gvl(Some(ffi_wrap::<F, R>), arg_ptr);
        result.assume_init()
    }
}

unsafe extern "C" fn ffi_wrap<F, R>(arg: *mut c_void) -> *mut c_void
where
    F: FnMut() -> R,
    R: Sized,
{
    let arg = arg as *mut (&mut F, &mut MaybeUninit<R>);
    let (func, result) = unsafe { &mut *arg };
    result.write(func());

    null_mut()
}

unsafe extern "C" fn ffi_wrap_interrupt<F>(arg: *mut c_void)
where
    F: FnMut(),
{
    let arg = arg as *mut (&mut F,);
    let (func,) = unsafe { &mut *arg };
    func();
}
