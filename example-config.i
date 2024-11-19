before_fmt "(( "

## available fmt values: desktop, focused, occupied, reversed, icon, window_class
## just like in rust, curly braces are escaped with another curly brace
fmt "{desktop} {focused} {occupied} {reversed} {icon} {color};  "

after_fmt ")) "

title ".*Reddit.*"   
title ".*Stack Overflow.*" 
title ".*YouTube.*"   focused_color #890
class "firefox"    color #42069 focused_color #2137
class "discord" 
class "steam" 
class "kitty"   color #500 reversed
class "mpv"  reversed
class "steam_app*" 󰊗

empty     color #000 focused_color #000
default   color #000 focused_color #000
