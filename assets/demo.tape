Output assets/demo.gif

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Catppuccin Mocha"
Set Padding 20
Set TypingSpeed 60ms

# Start envi with example directory
Type "envi example/"
Enter
Sleep 2s

# Navigate through variables
Sleep 1s
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "j"
Sleep 1s

# Switch to another profile
Type "l"
Sleep 1.5s
Type "l"
Sleep 1.5s

# Go back to first profile
Type "h"
Sleep 500ms
Type "h"
Sleep 1s

# Open diff view
Type "d"
Sleep 2.5s

# Cycle diff target
Tab
Sleep 2s

# Close diff
Escape
Sleep 1s

# Open code scan
Type "s"
Sleep 2.5s

# Close scan
Escape
Sleep 1s

# Search for a variable
Type "/"
Sleep 500ms
Type "API"
Sleep 1.5s
Enter
Sleep 1s

# Edit a value
Type "e"
Sleep 500ms
Type "new-value-here"
Sleep 1s
Escape
Sleep 1s

# Show help
Type "?"
Sleep 2.5s
Escape
Sleep 1s

# Quit
Type "q"
Sleep 500ms
