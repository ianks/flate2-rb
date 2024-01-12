# frozen_string_literal: true

require "benchmark/ips"
require "securerandom"
require "flate2"
require "zlib"

DATA = SecureRandom.random_bytes(1024 * 1024)
COMPRESSED = Flate2.gzip(DATA)

Benchmark.ips do |x|
  x.report("Flate2.gunzip") do
    Flate2.gunzip(COMPRESSED)
  end

  unless ENV["PROFILE_MODE"] == "1"
    x.report("Zlib.gunzip") do
      Zlib.gunzip(COMPRESSED)
    end

    x.compare!
  end
end
