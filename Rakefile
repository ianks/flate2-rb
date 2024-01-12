# frozen_string_literal: true

require "bundler/gem_tasks"
require "minitest/test_task"

Minitest::TestTask.create

require "rubocop/rake_task"

RuboCop::RakeTask.new

require "rb_sys/extensiontask"

GEMSPEC = Gem::Specification.load("flate2.gemspec")

RbSys::ExtensionTask.new("flate2", GEMSPEC)

task build: :compile
task default: %i[compile test rubocop]
