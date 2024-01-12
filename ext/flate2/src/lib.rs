mod nogvl;
mod tmplock;

use std::io::Write;

use magnus::{
    exception::standard_error,
    function,
    prelude::*,
    value::{InnerValue, Lazy},
    Error, ExceptionClass, RModule, RString, Ruby,
};
use tmplock::Tmplock;

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

fn gzip(ruby: &Ruby, data: RString) -> Result<RString, Error> {
    let (buf, _guard) = unsafe { data.as_locked_slice()? };
    let out = RString::buf_new(64 * 1024);

    nogvl::nogvl(|| {
        let mut encoder = flate2_rs::write::GzEncoder::new(out, flate2_rs::Compression::best());

        encoder
            .write_all(buf)
            .map_err(|e| Error::new(ENCODE_ERROR.get_inner_with(ruby), e.to_string()))?;

        encoder
            .finish()
            .map_err(|e| Error::new(ENCODE_ERROR.get_inner_with(ruby), e.to_string()))
    })
}

fn gunzip(ruby: &Ruby, data: RString) -> Result<RString, Error> {
    let (buf, _guard) = unsafe { data.as_locked_slice()? };
    let mut out = RString::buf_new(64 * 1024);

    nogvl::nogvl(|| {
        let mut decoder = flate2_rs::read::GzDecoder::new(buf);

        std::io::copy(&mut decoder, &mut out)
            .map_err(|e| Error::new(DECODE_ERROR.get_inner_with(ruby), e.to_string()))
    })?;

    Ok(out)
}

#[magnus::init(name = "flate2")]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Flate2")?;
    module.define_singleton_method("gzip", function!(gzip, 1))?;
    module.define_singleton_method("gunzip", function!(gunzip, 1))?;
    Ok(())
}
