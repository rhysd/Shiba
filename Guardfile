def which cmd
  dir = ENV['PATH'].split(':').find {|p| File.executable? File.join(p, cmd)}
  File.join(dir, cmd) unless dir.nil?
end

$has_terminal_notifier = which 'terminal-notifier'
$has_notify_send = which 'notify-send'
$has_tmux = which 'tmux'

def notify file
  msg = "'#{file} failed\n#{Time.now.to_s}'"
  case
  when $has_terminal_notifier
    `terminal-notifier -message #{msg}`
  when $has_notify_send
    `notify-send #{msg}`
  when $has_tmux
    `tmux display-message #{msg}` if system('tmux list-clients 1>/dev/null 2>&1')
  else
    puts "FAIL: #{msg}"
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

def npm_exe(f, cmd, *args)
  execute(f, "./node_modules/.bin/#{cmd}", *args)
end

def tsc(f, dir)
  npm_exe(f, 'tsc', '-p', dir)
end

def mocha(f, path)
  npm_exe(f, 'mocha', '--exit', '--require', 'intelli-espower-loader', path)
end

def slimrb(input, output)
  execute(input, 'slimrb', input, output)
end

ignore /^node_modules/, /^build/, /^bower_components/

guard :shell do
  watch /^.+\.ts/ do |m|
    dir = File.dirname m[0]
    case dir
    when 'browser', 'renderer'
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
