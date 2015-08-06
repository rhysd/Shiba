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

task :dep do
  sh 'npm install --dev'
  sh 'bower install'
end

task :build_slim do
  ensure_cmd 'slimrb'
  mkdir_p 'build/static'

  Dir['static/*.slim'].each do |slim_file|
    sh "slimrb #{slim_file} build/static/#{File.basename(slim_file, '.slim')}.html"
  end
end

task :build_test do
  ensure_cmd 'tsc'
  ensure_cmd 'bower'
  sh 'tsc -p tests/browser'
  cd 'tests/runner' do
    sh 'bower install'
  end
end

task :build_typescript do
  ensure_cmd 'tsc'
  ensure_cmd 'tsd'

  sh 'tsd install'
  sh 'tsc -p src/browser'
  sh 'tsc src/renderer/*.ts --out build/src/renderer/index.js'
end

task :build => %i(build_slim build_typescript)

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
