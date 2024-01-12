# frozen_string_literal: true

require "benchmark/ips"
require "securerandom"
require "flate2"
require "zlib"

DATA = SecureRandom.random_bytes(1024 * 1024)

Benchmark.ips do |x|
  x.report("Flate2.gzip") do
    Flate2.gzip(DATA)
  end

  unless ENV["PROFILE_MODE"] == "1"
    x.report("Zlib.gzip") do
      Zlib.gzip(DATA, level: 9)
    end

    x.compare!
  end
end
