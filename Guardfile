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
  else
    puts "OK"
  end
end

ignore /^node_modules/, /^build/, /^typings/, /^bower_components/

guard :shell do
  watch /^.+\.ts/ do |m|
    dir = File.dirname m[0]
    case File.basename(dir)
    when 'browser'
      execute(m[0], 'tsc', '-p', dir)
    when 'renderer'
      execute(m[0], 'tsc', *Dir['src/renderer/*.ts'], '--out', 'build/src/renderer/index.js')
    end
  end

  watch /^.+\.slim/ do |m|
    if File.basename(File.dirname m[0]) == 'static'
      execute(m[0], 'slimrb', m[0], "build/static/#{File.basename(m[0], '.slim')}.html")
    end
  end
end
