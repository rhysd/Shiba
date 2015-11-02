require 'fileutils'
include FileUtils

BIN_DIR = './node_modules/.bin'.freeze

def cmd_exists?(cmd)
  File.exists?(cmd) && File.executable?(cmd)
end

def ensure_cmd(cmd)
  $cmd_cache ||= []
  return true if $cmd_cache.include? cmd

  paths = ENV['PATH'].split(':').uniq
  unless paths.any?{|p| cmd_exists? "#{p}/#{cmd}" }
    raise "'#{cmd}' command doesn't exist"
  else
    $cmd_cache << cmd
  end
end

file "node_modules" do
  ensure_cmd 'npm'
  sh 'npm install --dev'
end

file "bower_components" do
  ensure_cmd 'bower'
  sh 'bower install'
end

task :dep => %i(node_modules bower_components)

task :build_slim do
  ensure_cmd 'slimrb'
  directory 'build/static'

  Dir['static/*.slim'].each do |slim_file|
    sh "slimrb #{slim_file} build/static/#{File.basename(slim_file, '.slim')}.html"
  end
end

task :build_test do
  ensure_cmd 'tsc'
  ensure_cmd 'bower'
  sh 'tsc -p tests/browser'
  sh 'tsc tests/renderer/*.ts --out tests/renderer/index.js'
  cd 'tests/runner' do
    sh 'bower install'
  end
end

file "typings" do
  ensure_cmd 'tsd'
  sh 'tsd install'
end

task :build_typescript => %i(typings) do
  ensure_cmd 'tsc'
  sh 'tsc -p src/browser'
  sh 'tsc -p src/renderer'
end

task :build => %i(dep build_slim build_typescript)

task :npm_publish => %i(build) do
  mkdir 'npm-publish'
  %w(bower.json package.json build bin README.md).each{|p| cp_r p, 'npm-publish' }
  cd 'npm-publish' do
    sh 'bower install --production'
    sh 'npm install --save electron-prebuilt'
    sh 'npm publish'
  end
  rm_rf 'npm-publish'
end

task :test => %i(build_test) do
  sh 'tsc -p tests/browser'
  sh 'tsc tests/renderer/keyboard_test.ts --out tests/renderer/index.js'
  sh 'npm test'
end

task :asar => %i(build) do
  raise "'asar' command doesn't exist" unless cmd_exists? "#{BIN_DIR}/asar"

  mkdir_p 'archive'
  begin
    %w(bower.json package.json build).each{|p| cp_r p, 'archive' }
    cd 'archive' do
      sh 'npm install --production'
      sh 'bower install --production'
    end
    sh "#{BIN_DIR}/asar pack archive app.asar"
  ensure
    rm_rf 'archive'
  end
end

task :run => %i(dep asar) do
  sh "#{BIN_DIR}/electron app.asar README.md"
end

task :clean do
  %w(npm-publish build/src build/static archive).each{|tmpdir| rm_rf tmpdir}
end
