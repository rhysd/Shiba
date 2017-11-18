#!/usr/bin/env ruby

require 'fileutils'

$LOAD_PATH << './gemoji/lib'

require 'emoji'

Dir.mkdir 'gemoji/tmp'

Emoji.all.each do |emoji|
  raw = emoji.raw
  next unless raw
  code = raw.unpack('*U').map{|c| c.to_s(16) }.join('-')
  png = "gemoji/public/images/emoji/unicode/#{code}.png"
  if File.exists? png
    FileUtils.cp(png, "gemoji/tmp/#{emoji.name}.png")
    puts "copied: #{emoji.name} (#{code})"
  else
    puts "not found: #{emoji.name}"
  end
end
