require 'fileutils'
include FileUtils

PREFIX = `npm prefix`.chomp
BIN_DIR = "#{PREFIX}/node_modules/.bin"

Dir.chdir PREFIX

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
  mkdir_p "archive"
  %w(bower.json package.json build).each{|p| cp_r p, 'archive' }
  cd 'archive' do
    sh 'npm install --production'
    sh 'bower install --production'
    sh 'npm uninstall electron-prebuilt'
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

file "typings" do
  raise "'typings' command doesn't exist" unless cmd_exists? "#{BIN_DIR}/typings"
  sh "#{BIN_DIR}/typings install"
end

task :dep => [:node_modules, :bower_components, :typings]

task :build_slim do
  ensure_cmd 'slimrb'
  mkdir_p 'build/static'

  Dir['renderer/*.slim'].each do |slim_file|
    sh "slimrb #{slim_file} build/static/#{File.basename(slim_file, '.slim')}.html"
  end
end

task :build_typescript => [:typings] do
  ensure_cmd 'tsc'
  mkdir_p 'build/src/renderer'
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
  ver = ENV['ELECTRON']
  release "--platform=darwin --arch=x64 --version=#{ver} --asar --icon=../resource/image/icon/shibainu.icns"
  release "--platform=win32 --arch=ia32 --version=#{ver} --asar --icon=./resource/image/icon/shibainu.ico"
  release "--platform=win32 --arch=x64 --version=#{ver} --asar --icon=./resource/image/icon/shibainu.ico"
  release "--platform=linux --arch=ia32 --version=#{ver} --asar --icon=./resource/image/icon/shibainu.ico"
  release "--platform=linux --arch=x64 --version=#{ver} --asar --icon=./resource/image/icon/shibainu.ico"
end

task :build_test do
  ensure_cmd 'tsc'
  sh 'tsc -p test/main'
end

task :test => [:build_test] do
  sh "#{BIN_DIR}/mocha test/main/test/main"
end

task :lint do
  ensure_cmd 'tslint'
  ts = `git ls-files`.split("\n").select{|p| p =~ /.ts$/}.join(' ')
  sh "tslint #{ts}"
end

task :clean do
  %w(npm-publish build/src build/static archive).each{|tmpdir| rm_rf tmpdir}
end

task :watch do
  sh 'guard --watchdir browser renderer typings test'
end
