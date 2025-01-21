# frozen_string_literal: true

task "demo" do
  require "json"

  sh "vhs demo.tape"
  puts "Post-processing demo_raw.cast"

  lines = File.readlines("./demo_raw.cast")
  header = JSON.parse(lines[0])
  events_parsed = lines[1..].map { |line| JSON.parse(line) }
  introducing_index =
    events_parsed.find_index { |event| event[2].include?("Introducing") }
  ps1_index =
    events_parsed[..introducing_index].rindex do |event|
      event[2].include?("ccsum")
    end
  offset_time = events_parsed[ps1_index][0]

  events_shifted =
    events_parsed.filter_map do |event|
      if event[0] < offset_time
        nil
      else
        event[0] -= offset_time
        event
      end
    end

  File.write("./demo.cast", [header, *events_shifted].map(&:to_json).join("\n"))

  sh 'agg --font-family "JetBrainsMono Nerd Font Mono" demo.cast demo.gif'
end

task "readme" do
  readme = File.read("README.md")
  usage = `cargo run -- --help`.strip
  readme.gsub!(/(?<=<!-- usage starts here -->\n).*(?=<!-- usage ends here)/m) do
    <<~USAGE
      ```
      #{usage}
      ```
    USAGE
  end
  File.write("README.md", readme)
end
