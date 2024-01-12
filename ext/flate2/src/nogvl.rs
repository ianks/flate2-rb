use std::{ffi::c_void, mem::MaybeUninit, ptr::null_mut};

use rb_sys::rb_thread_call_without_gvl;

#[derive(Debug, thiserror::Error)]
pub enum InterruptableError {
    #[error("interrupted")]
    Interrupt,
    #[error(transparent)]
    Io(#[from] std::io::Error),
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
            Some(call_without_gvl::<F, R>),
            arg_ptr,
            Some(call_without_gvl_interrupt_callback::<I>),
            interrupt_ptr,
        );
        result.assume_init()
    }
}

unsafe extern "C" fn call_without_gvl<F, R>(arg: *mut c_void) -> *mut c_void
where
    F: FnMut() -> R,
    R: Sized,
{
    let arg = arg as *mut (&mut F, &mut MaybeUninit<R>);
    let (func, result) = unsafe { &mut *arg };
    result.write(func());

    null_mut()
}

unsafe extern "C" fn call_without_gvl_interrupt_callback<F>(arg: *mut c_void)
where
    F: FnMut(),
{
    let arg = arg as *mut (&mut F,);
    let (func,) = unsafe { &mut *arg };
    func();
}
