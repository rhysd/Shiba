require 'fileutils'
include FileUtils

BIN_DIR = './node_modules/.bin'.freeze

def cp_paths(paths, dest)
  paths.each do |p|
    cp_r p, dest
  end
end

def cmd_exists?(cmd)
  File.exists?(cmd) && File.executable?(cmd)
end

def ensure_cmd(cmd)
  paths = ENV['PATH'].split(':').uniq
  raise "'#{cmd}' command doesn't exist" unless paths.any?{|p| cmd_exists? "#{p}/#{cmd}" }
end

task :dep do
  system 'npm install'
  system 'bower install'
end

task :npm_publish do
  mkdir 'npm-publish'
  %w(bower.json package.json src build bin README.md).each{|p| cp_r p, 'npm-publish' }
  cd 'npm-publish' do
    system 'bower install --production'
    system 'npm install --save electron-prebuilt'
    system 'npm publish'
  end
  rm_rf 'npm-publish'
end

task :build_slim do
  ensure_cmd 'slimrb'
  mkdir_p 'build/static'

  Dir['static/*.slim'].each do |slim_file|
    puts "converting #{slim_file}"
    system "slimrb #{slim_file} build/static/#{File.basename(slim_file, '.slim')}.html"
  end
end

task :build_typescript do
  ensure_cmd 'tsc'
  ensure_cmd 'tsd'

  puts 'installing typings/**/*.d.ts'
  system 'tsd install'

  puts 'compiling src/browser/*.ts'
  system 'tsc -p src/browser'
  puts 'compiling src/renderer/*.ts'
  system 'tsc src/renderer/*.ts --out build/src/renderer/index.js'
end

task :build => [:build_slim, :build_typescript]

task :asar do
  raise "'asar' command doesn't exist" unless cmd_exists? "#{BIN_DIR}/asar"

  mkdir_p 'archive/resource/image'
  %w(bower.json package.json src build).each{|p| cp_r p, 'archive' }
  cp_r 'resource/image/emoji', 'archive/resource/image/'
  cd 'archive' do
    system 'npm install --production'
    system 'bower install --production'
  end
  system "#{BIN_DIR}/asar pack archive app.asar"
  rm_rf 'archive'
end

task :clean do
  %w(npm-publish build archive).each{|tmpdir| rm_rf tmpdir}
end
