require 'json'
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

def npm_sh(cmd)
  sh "#{BIN_DIR}/#{cmd}"
end

file "node_modules" do
  ensure_cmd 'npm'
  sh 'npm install'
  npm_sh 'electron-rebuild'
end

file "bower_components" do
  npm_sh 'bower install'
end

task :dep => [:node_modules, :bower_components]

task :build_slim do
  ensure_cmd 'slimrb'
  mkdir_p 'build/static'

  Dir['renderer/*.slim'].each do |slim_file|
    sh "slimrb #{slim_file} build/static/#{File.basename(slim_file, '.slim')}.html"
  end
end

task :build_typescript do
  mkdir_p 'build/src/renderer'
  npm_sh 'tsc --pretty -p ./browser'
  npm_sh 'tsc --pretty -p ./renderer'
end

task :compile => [:build_slim, :build_typescript]

task :build => [:dep, :compile]

task :prepare_release => [:build] do
  mkdir_p "archive"
  %w(bower.json package.json build).each{|p| cp_r p, 'archive' }
  cd 'archive' do
    sh 'npm install --production'
    sh 'npm uninstall --production electron'
    sh 'npm prune --production electron'
    sh 'npm install --production'
    sh '../node_modules/.bin/bower install --production'
    sh '../node_modules/.bin/electron-rebuild'
  end
end

task :package do
  mkdir_p 'packages'
  def release(options)
    cd 'archive' do
      npm_sh "electron-packager ./ Shiba #{options.join ' '} --no-prune"
      Dir['Shiba-*'].each do |dst|
        cp_r '../README.md', dst
        cp_r '../docs', dst
        sh "zip --symlinks #{dst}.zip -r #{dst}"
        mv "#{dst}.zip", '../packages'
        rm_r dst
      end
    end
  end

  electron_json = JSON.load(File.open('node_modules/electron/package.json'))
  electron_ver = electron_json['version']
  app_json = JSON.load(File.open('package.json'))
  ver = app_json['version']
  release %W(
    --platform=darwin
    --arch=x64
    --electron-version=#{electron_ver}
    --asar
    --icon=../resource/image/icon/shibainu.icns
    --app-version='#{ver}'
    --build-version='#{ver}'
  )
  release %W(
    --platform=win32
    --arch=ia32
    --electron-version=#{electron_ver}
    --asar
    --icon=../resource/image/icon/shibainu.ico
    --app-version='#{ver}'
    --build-version='#{ver}'
  )
  release %W(
    --platform=win32
    --arch=x64
    --electron-version=#{electron_ver}
    --asar
    --icon=../resource/image/icon/shibainu.ico
    --app-version='#{ver}'
    --build-version='#{ver}'
  )
  release %W(
    --platform=linux
    --arch=ia32
    --electron-version=#{electron_ver}
    --asar
    --icon=../resource/image/icon/shibainu.ico
    --app-version='#{ver}'
    --build-version='#{ver}'
  )
  release %W(
    --platform=linux
    --arch=x64
    --electron-version=#{electron_ver}
    --asar
    --icon=../resource/image/icon/shibainu.ico
    --app-version='#{ver}'
    --build-version='#{ver}'
  )
end

task :release => [:prepare_release, :package]

task :build_test do
  npm_sh 'tsc --pretty -p test/main'
  npm_sh 'tsc --pretty -p test/renderer'
end

task :test => [:build_test] do
  npm_sh 'mocha --exit --require intelli-espower-loader test/main/test/main test/renderer/test/renderer'
end

task :build_e2e do
  npm_sh 'tsc --pretty -p test/e2e'
end

task :e2e => [:build_e2e] do
  npm_sh 'mocha --exit test/e2e --opts test/e2e/mocha.opts'
end

task :lint do
  npm_sh 'tslint --project browser/'
  npm_sh 'tslint --project renderer/'
end

task :clean do
  %w(build/src build/static archive).each{|tmpdir| rm_rf tmpdir}
end

task :watch do
  sh 'guard --watchdir browser renderer test'
end

task :update_emojis do
  sh './scripts/update_emoji.sh'
end
