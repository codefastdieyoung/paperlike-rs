# Paperlike Control Utility

A command-line tool to control Paperlike e-ink monitors via USB serial interface.

**Note:** Works and has been tested with Dasung Paperlike 13K (color) 37hz version of the monitor. Other monitor models has not been tested but it is expected that some of the commands should also work with 253 monitor and other monitors from revolutionary series. If you have different Dasung monitor model - use at your own risk.

## Install

**Requirements:** rust, cargo

Build the project and install binary to /usr/local/bin:

```
cargo build --release
sudo cp target/release/paperlike /usr/local/bin/
sudo chmod +x /usr/local/bin/paperlike
```

**Optional:** Create udev rule for the default main user group and create /dev/paperlike symlink (avoids needing sudo):

```bash
echo 'SUBSYSTEM=="tty", ATTRS{idVendor}=="1a86", ATTRS{idProduct}=="7523", MODE="0666", GROUP="'$(id -gn)'", SYMLINK+="paperlike"' | sudo tee /etc/udev/rules.d/99-paperlike.rules > /dev/null
# reload udev:
sudo udevadm control --reload-rules
sudo udevadm trigger
```

# Usage

If udev rules have been created you can use /dev/paperlike path without sudo (e.g. in scripts or keyboard bindings in your desktop environment). Otherwise, use sudo with the correct device path like /dev/ttyUSB0.

```
# Detect device (optional - to find correct /dev/ttyUSB*)  
paperlike  # or: sudo paperlike (if no udev rule)

# With udev rule it can be used without sudo
paperlike /dev/paperlike -i

# Without udev rule
sudo paperlike /dev/ttyUSB0 -i

# Summary:
paperlike /dev/paperlike -h    # Help
paperlike /dev/paperlike -i    # Get current parameters
paperlike /dev/paperlike -r    # Refresh screen
paperlike /dev/paperlike -c 4  # Set contrast to 4
paperlike /dev/paperlike -t 5  # Set temperature to 5
paperlike /dev/paperlike -f 5  # Set frontlight to 5
paperlike /dev/paperlike -m 2  # Set mode to 2
```

# Utils

A helper script paperlike-utils.sh included - it provides utilities to set the monitor
orientation, touch area and shortcuts for setting visual appearance using paperlike cli.
A snippet in the header is provided for i3wm to conveniently control your monitor.
