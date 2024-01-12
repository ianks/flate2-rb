# frozen_string_literal: true

require "bundler/gem_tasks"
require "minitest/test_task"

Minitest::TestTask.create

require "rubocop/rake_task"

RuboCop::RakeTask.new

require "rb_sys/extensiontask"

GEMSPEC = Gem::Specification.load("flate2.gemspec")

RbSys::ExtensionTask.new("flate2", GEMSPEC) do |ext|
  ext.lib_dir = "lib/flate2"
end

task build: :compile
task default: %i[compile test rubocop]

Dir["bench/*.rb"].each do |bench|
  desc "Run the #{bench} benchmark"
  task "bench:#{bench}" => "compile:release" do
    ruby "-Ilib", bench
  end

  desc "Profile the #{bench} benchmark"
  task "profile:#{bench}" => "compile:release" do
    ENV["PROFILE_MODE"] = "1"
    sh "samply", "record", RbConfig.ruby, "-Ilib", bench
    ENV["PROFILE_MODE"] = nil
  end

  task bench: "bench:#{bench}"
  task profile: "profile:#{bench}"
end
