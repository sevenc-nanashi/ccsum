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
Type "# But it would be better if files with the same name were grouped together"
Sleep 1s
Enter
Sleep 500ms
Type "# Let's use the --group option to group files with the same name"
Sleep 1s
Enter
Sleep 500ms
Up@100ms 4
Sleep 300ms
Type " --group basename # or -g basename"
Sleep 1s
Enter
Sleep 500ms
Type "# Now the output is grouped by the basename of the files"
Sleep 1s
Enter
Sleep 500ms
Type "# Download ccsum now and give it a try!"
Sleep 2s
Ctrl+D
