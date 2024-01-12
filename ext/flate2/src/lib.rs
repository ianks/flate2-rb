mod nogvl;
mod tmplock;

use std::{
    cell::Cell,
    io::{Read, Write},
};

use magnus::{
    exception::standard_error,
    function,
    prelude::*,
    value::{InnerValue, Lazy},
    Error, ExceptionClass, RModule, RString, Ruby,
};
use nogvl::InterruptableError;
use tmplock::Tmplock;

const CHUNK_SIZE: usize = 1024;

static ROOT_MODULE: Lazy<RModule> = Lazy::new(|ruby| ruby.define_module("Flate2").unwrap());

static ERROR: Lazy<ExceptionClass> = Lazy::new(|ruby| {
    ROOT_MODULE
        .get_inner_with(ruby)
        .define_error("Error", standard_error())
        .unwrap()
});

static ENCODE_ERROR: Lazy<ExceptionClass> = Lazy::new(|ruby| {
    ROOT_MODULE
        .get_inner_with(ruby)
        .define_error("EncodeError", ERROR.get_inner_with(ruby))
        .unwrap()
});

static DECODE_ERROR: Lazy<ExceptionClass> = Lazy::new(|ruby| {
    ROOT_MODULE
        .get_inner_with(ruby)
        .define_error("DecodeError", ERROR.get_inner_with(ruby))
        .unwrap()
});

fn gunzip(ruby: &Ruby, data: RString) -> Result<RString, Error> {
    let (buf, _guard) = unsafe { data.as_locked_slice()? };
    let mut out = ruby.str_buf_new(0);

    let mut decoder = flate2_rs::read::GzDecoder::new(buf);
    let interrupt_flag = Cell::new(false);
    let interrupt_callback = || interrupt_flag.set(true);
    let mut total = 0;

    let mut func = || loop {
        if interrupt_flag.get() {
            return Err(InterruptableError::Interrupt);
        }

        let mut buf = [0; CHUNK_SIZE];
        let read = decoder.read(&mut buf)?;

        if read == 0 {
            return Ok(());
        }

        total += read;

        out.write_all(&buf[..read])?
    };

    loop {
        match nogvl::nogvl_with_interrupt_callback(&mut func, interrupt_callback) {
            Ok(_) => break,
            Err(InterruptableError::Interrupt) => {
                interrupt_flag.set(false);
            }
            Err(e) => return Err(Error::new(DECODE_ERROR.get_inner_with(ruby), e.to_string())),
        };
    }

    Ok(out)
}

fn gzip(ruby: &Ruby, data: RString) -> Result<RString, Error> {
    let (buf, _guard) = unsafe { data.as_locked_slice()? };
    let out = ruby.str_buf_new(0);
    let mut cursored_input = std::io::Cursor::new(buf);
    let interrupt_flag = Cell::new(false);
    let mut encoder = flate2_rs::write::GzEncoder::new(out, flate2_rs::Compression::best());
    let mut interrupt_callback = || interrupt_flag.set(true);
    let mut total = 0;

    let mut func = || loop {
        if interrupt_flag.get() {
            interrupt_flag.set(false);
            return Err(InterruptableError::Interrupt);
        }

        let mut buf = [0; CHUNK_SIZE];
        let read = cursored_input.read(&mut buf)?;

        if read == 0 {
            return Ok(());
        }

        total += read;

        encoder.write_all(&buf[..read])?
    };

    loop {
        match nogvl::nogvl_with_interrupt_callback(&mut func, &mut interrupt_callback) {
            Ok(_) => break,
            Err(InterruptableError::Interrupt) => {}
            Err(e) => return Err(Error::new(ENCODE_ERROR.get_inner_with(ruby), e.to_string())),
        };
    }

    encoder
        .finish()
        .map_err(|e| Error::new(ENCODE_ERROR.get_inner_with(ruby), e.to_string()))
}

#[magnus::init(name = "flate2")]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Flate2")?;
    module.define_singleton_method("gzip", function!(gzip, 1))?;
    module.define_singleton_method("gunzip", function!(gunzip, 1))?;
    Ok(())
}
