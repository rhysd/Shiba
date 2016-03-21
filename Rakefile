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

def make_archive_dir
  mkdir_p 'archive'
  %w(bower.json package.json build).each{|p| cp_r p, 'archive' }
  cd 'archive' do
    sh 'npm install --production'
    sh 'bower install --production'
  end
end

file "node_modules" do
  ensure_cmd 'npm'
  sh 'npm install'
end

file "bower_components" do
  ensure_cmd 'bower'
  sh 'bower install'
end

task :dep => [:node_modules, :bower_components]

task :build_slim do
  ensure_cmd 'slimrb'
  directory 'build/static'

  Dir['renderer/*.slim'].each do |slim_file|
    sh "slimrb #{slim_file} build/static/#{File.basename(slim_file, '.slim')}.html"
  end
end

task :build_test do
  ensure_cmd 'tsc'
  ensure_cmd 'bower'
  sh 'tsc -p tests/browser'
  sh 'tsc tests/renderer/*.ts --out tests/renderer/index.js'
end

file "typings" do
  raise "'typings' command doesn't exist" unless cmd_exists? "#{BIN_DIR}/typings"
  sh "#{BIN_DIR}/typings install"
end

task :build_typescript => [:typings] do
  ensure_cmd 'tsc'
  sh 'tsc -p ./browser'
  sh 'tsc -p ./renderer'
end

task :build => [:dep, :build_slim, :build_typescript]

task :npm_publish => [:build] do
  mkdir 'npm-publish'
  %w(bower.json package.json build bin README.md).each{|p| cp_r p, 'npm-publish' }
  cd 'npm-publish' do
    sh 'bower install --production'
    sh 'npm install --save electron-prebuilt'
    sh 'npm publish'
  end
  rm_rf 'npm-publish'
end

task :test => [:build_test] do
  sh 'tsc -p tests/browser'
  sh 'tsc tests/renderer/keyboard_test.ts --out tests/renderer/index.js'
  sh 'npm test'
end

task :asar => [:build] do
  raise "'asar' command doesn't exist" unless cmd_exists? "#{BIN_DIR}/asar"

  begin
    make_archive_dir
    sh "#{BIN_DIR}/asar pack archive app.asar"
  ensure
    rm_rf 'archive'
  end
end

task :release => [:build] do
  ensure_cmd 'electron-packager'
  make_archive_dir
  mkdir_p 'packages'
  def release(options)
    cd 'archive' do
      sh "electron-packager ./ Shiba #{options}"
      Dir['Shiba-*'].each do |dst|
        cp_r '../README.md', dst
        cp_r '../docs', dst
        sh "zip --symlinks #{dst}.zip -r #{dst}"
        mv "#{dst}.zip", '../packages'
        rm_r dst
      end
    end
  end
  release '--platform=darwin --arch=x64 --version=0.36.5 --asar --icon=../resource/image/icon/shibainu.icns'
  release '--platform=win32 --arch=ia32 --version=0.36.5 --asar --icon=./resource/image/icon/shibainu.ico'
  release '--platform=win32 --arch=x64 --version=0.36.5 --asar --icon=./resource/image/icon/shibainu.ico'
  release '--platform=linux --arch=ia32 --version=0.36.5 --asar --icon=./resource/image/icon/shibainu.ico'
  release '--platform=linux --arch=x64 --version=0.36.5 --asar --icon=./resource/image/icon/shibainu.ico'
end

task :clean do
  %w(npm-publish build/src build/static archive).each{|tmpdir| rm_rf tmpdir}
end

task :watch do
  sh 'guard --watchdir static browser renderer tests typings'
end
