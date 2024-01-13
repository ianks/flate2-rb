# frozen_string_literal: true

require "benchmark/ips"
require "securerandom"
require "flate2"
require "zlib"

GEMSPEC = Gem::Specification.load("flate2.gemspec")
FILES = GEMSPEC.files + Dir["lib/**/*.{bundle,so}"]
DATASET = FILES.map { |file| Zlib.gzip(File.binread(file)) }

Benchmark.ips do |x|
  x.config(warmup: 0)

  x.report("Flate2.gunzip") do
    DATASET.each do |data|
      Flate2.gunzip(data)
    end
  end

  x.report("Zlib.gunzip") do
    DATASET.each do |data|
      Zlib.gunzip(data)
    end
  end

  x.compare!
end
