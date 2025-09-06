#!/bin/bash

# Helper utilities to set up monitor orientation, touch area, appearance, refresh and use
# it in other automation scripts (e.g. configure desktop environment keybindings).

# The transformation matrix should be calculated considering native display resolution
# and positioning of paperlike display (e.g. extended left/right).
# Carefully review https://wiki.archlinux.org/title/Calibrating_Touchscreen to understand
# how to calculate the matrix transform.

# You may also need to change the device name.
# To find touch device: xinput list | grep -i touch
# Replace "wch.cn USB2IIC_CTP_CONTROL" with your device name

# To use this script in i3wm to easily control your monitor with bindings menu:
# cp paperlike-utils.sh ~/.local/bin/
# ~/.config/i3/config:
# set $mode_display Paperlike: [c]refresh, [v]isuals, [i]nverted, [n]ormal, [l]eft, [r]ight
# bindsym $mod+i mode "$mode_display"
# mode "$mode_display" {
#     bindsym c exec --no-startup-id paperlike-utils.sh refresh
#     bindsym v exec --no-startup-id paperlike-utils.sh visuals
#     bindsym i exec --no-startup-id paperlike-utils.sh inverted
#     bindsym n exec --no-startup-id paperlike-utils.sh normal
#     bindsym l exec --no-startup-id paperlike-utils.sh left
#     bindsym r exec --no-startup-id paperlike-utils.sh right
#     bindsym Control+g mode "default"
#     bindsym q mode "default"
#     bindsym Escape mode "default"
#     bindsym Return mode "default"
# }

set -e

subcommand="$1"

shift || (echo "Please specify one of: off, normal, inverted, left, visuals, refresh" && exit)

case $subcommand in
  "off")
    xrandr --output DP-2 --off
    ;;
  "normal")
    # For NORMAL (paperlike right): x_scale=3200/5760=0.555555, x_offset=2560/5760=0.444444
    # Matrix: [0.555555, 0, 0.444444, 0, 1, 0, 0, 0, 1]
    xrandr --output DP-2 --right-of eDP-1 --primary --rotate normal --auto
    xinput set-prop "wch.cn USB2IIC_CTP_CONTROL" --type=float "Coordinate Transformation Matrix" 0.555555 0 0.444444 0 1 0 0 0 1
    ;;
  "inverted")
    # for INVERTED: negate scales, adjust offsets: [-0.555555, 0, 1, 0, -1, 1, 0, 0, 1]
    xrandr --output DP-2 --right-of eDP-1 --primary --rotate inverted --auto
    xinput set-prop "wch.cn USB2IIC_CTP_CONTROL" --type=float "Coordinate Transformation Matrix" -0.555555 0 1 0 -1 1 0 0 1
    ;;
  "left")
    xrandr --output DP-2 --left-of eDP-1 --primary --rotate left --auto
    xinput set-prop "wch.cn USB2IIC_CTP_CONTROL" --type=float "Coordinate Transformation Matrix" 0 -0.484 0.484 1 0 0 0 0 1
    ;;
  "right")
    xrandr --output DP-2 --right-of eDP-1 --primary --rotate right --auto
    xinput set-prop "wch.cn USB2IIC_CTP_CONTROL" --type=float "Coordinate Transformation Matrix" 0 0.484 0.516 -1 0 1 0 0 1
    ;;
  "refresh")
    paperlike /dev/paperlike -r
    ;;
  "visuals")
    sleep 0.5  # short sleep between display settings to make sure it sets well
    paperlike /dev/paperlike -c 4  # contrast
    sleep 0.5
    paperlike /dev/paperlike -m 1  # mode
    sleep 0.5
    paperlike /dev/paperlike -t 4  # temperature
    sleep 0.5
    paperlike /dev/paperlike -f 2  # frontlight
    sleep 1
    paperlike /dev/paperlike -i    # info
    ;;
esac
