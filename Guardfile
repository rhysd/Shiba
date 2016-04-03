def which cmd
  dir = ENV['PATH'].split(':').find {|p| File.executable? File.join(p, cmd)}
  File.join(dir, cmd) unless dir.nil?
end

def notify file
  msg = "'#{file} failed\n#{Time.now.to_s}'"
  case
  when which('terminal-notifier')
    `terminal-notifier -message #{msg}`
  when which('notify-send')
    `notify-send #{msg}`
  when which('tmux')
    `tmux display-message #{msg}` if `tmux list-clients 1>/dev/null 2>&1` && $?.success?
  end
end

def execute(f, *args)
  print "#{f}: #{args.join(' ')}..."
  unless system(*args)
    puts "NG"
    notify f
    false
  else
    puts "OK"
    true
  end
end

def tsc(f, dir)
  execute(f, 'tsc', '-p', dir)
end

def mocha(f, path)
  execute(f, './node_modules/.bin/mocha', '--require', 'intelli-espower-loader', path)
end

def slimrb(input, output)
  execute(input, 'slimrb', input, output)
end

ignore /^node_modules/, /^build/, /^typings/, /^bower_components/

guard :shell do
  watch /^.+\.ts/ do |m|
    dir = File.dirname m[0]
    case dir
    when 'browser', 'renderer', 'tests/browser'
      tsc(m[0], dir)
    when 'test/main'
      tsc(m[0], dir) && mocha(m[0], "#{dir}/test/main/#{File.basename(m[0], '.ts')}.js")
    when 'test/renderer'
      tsc(m[0], dir) && mocha(m[0], "#{dir}/test/renderer/#{File.basename(m[0], '.ts')}.js")
    end
  end

  watch /^.+\.slim/ do |m|
    if File.dirname(m[0]) == 'renderer'
      slimrb(m[0], "build/static/#{File.basename(m[0], '.slim')}.html")
    end
  end
end
