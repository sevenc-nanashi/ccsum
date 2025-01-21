Require echo
Require ccsum
Require asciinema

Set Shell "bash"
Set FontSize 16
Set Width 1200
Set Height 600

Type@0s "asciinema rec demo_raw.cast --overwrite"
Enter
Sleep 3s
Type@0s "bleopt complete_auto_history="
Enter
Sleep 1s

Ctrl+L
Sleep 1s
Type "# Introducing ccsum, a command line utility for checksumming files"
Sleep 1s
Enter

Sleep 500ms
Type "# Let's compare with sha256sum, a classic checksum utility"
Sleep 1s
Enter

Sleep 500ms
Type "sha256sum ./demo/{foo,bar}/*.txt"
Sleep 1s
Enter

Sleep 500ms
Type "# Now let's use ccsum to checksum the same files"
Sleep 1s
Enter

Sleep 500ms
Type "ccsum ./demo/{foo,bar}/*.txt"
Sleep 1s
Enter

Sleep 500ms
Type "# The output is colored and easy to find the differences"
Sleep 1s
Enter

Sleep 500ms
Type "# There is also a --group option to group the output by the basename of the files"
Sleep 1s
Enter

Sleep 500ms
Up@100ms 3
Sleep 300ms
Type " --group # or -g"
Sleep 1s
Enter

Sleep 500ms
Type "# Now the output is grouped and easier to read"
Sleep 1s
Enter

Sleep 500ms
Type "# You can also use --group-with-check to check checksums in the same group"
Sleep 1s
Enter

Sleep 500ms
Up@100ms 3
Sleep 300ms
Ctrl+C
Type "F3-C--group-with-check # or -G"
Sleep 1s
Enter

Sleep 500ms
Type "# The command is failed (exit code 1) because demo/foo/01.txt and demo/bar/01.txt have different checksums"
Sleep 1s
Enter

Sleep 500ms
Type "# Download ccsum now and give it a try!"
Sleep 2s
Ctrl+D
