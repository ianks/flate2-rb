# frozen_string_literal: true

require "test_helper"
require "securerandom"

class TestFlate2 < Minitest::Test
  def test_roundtrip_small_string
    assert_equal("hello", Flate2.gunzip(Flate2.gzip("hello")))
  end

  def test_roundtrip_large_string
    content = SecureRandom.random_bytes(1024 * 1024 * 8)

    assert_equal(content, Flate2.gunzip(Flate2.gzip(content)))
  end

  def test_treats_string_as_binary
    result = Flate2.gzip("hello".encode(Encoding::UTF_16LE))

    refute_equal("hello", result)
    assert_equal("hello".encode(Encoding::UTF_16LE), Flate2.gunzip(result).force_encoding(Encoding::UTF_16LE))
  end

  def test_roundtrip_nil
    assert_raises(TypeError) { Flate2.gzip(nil) }
  end
end
