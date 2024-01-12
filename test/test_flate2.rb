# frozen_string_literal: true

require "test_helper"
require "securerandom"
require "zlib"

class TestFlate2 < Minitest::Test
  def test_roundtrip_nil
    assert_raises(TypeError) { Flate2.gzip(nil) }
  end

  def test_gzip_roundtrip_small_string
    assert_equal("hello", Flate2.gunzip(Flate2.gzip("hello")))
  end

  def test_gzip_roundtrip_large_string
    content = SecureRandom.random_bytes(1024 * 1024 * 8)

    assert_equal(content, Flate2.gunzip(Flate2.gzip(content)))
  end

  def test_gzip_treats_string_as_binary
    result = Flate2.gzip("hello".encode(Encoding::UTF_16LE))

    refute_equal("hello", result)
    assert_equal("hello".encode(Encoding::UTF_16LE), Flate2.gunzip(result).force_encoding(Encoding::UTF_16LE))
  end

  def test_gzip_thread_interrupts
    content = SecureRandom.random_bytes(1024 * 1024 * 8)
    stop = false
    ret = nil

    thr = Thread.new do
      ret = Flate2.gzip(content) until stop
    end

    10_000_000.times { thr.wakeup }
    stop = true
    thr.join

    assert_equal content, Zlib.gunzip(ret)
  end

  def test_gunzip_thread_interrupts
    content = SecureRandom.random_bytes(1024 * 1024 * 8)
    gzipped = Zlib.gzip(content)
    stop = false
    ret = nil

    thr = Thread.new do
      ret = Flate2.gunzip(gzipped) until stop
    end

    10_000_000.times { thr.wakeup }
    stop = true
    thr.join

    assert_equal content, ret
  end
end
