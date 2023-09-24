# mpris-player-info

This is my implementation of a mpris based interfaces for multiple bars I have used or considered to use in the last year.
The main difference compared to a pure playerctl based setup is that

1. the case that no player is currently active is detected
2. output can be easily hidden from within the interface
3. it does some additional processing.

The setup is highly opinionated but to support other outputs only the output format would b´have to be adapted.

## Components

### This crate

This serves as a library of code for working with mpris dbus services. It especially features a collection of streams that return the current state of the active mpris player.

### mpris-player-info

This is the main executable. Each of the tools corresponds to one sub command.

#### hide-server

This is a small dbus based server that only stores one property - wether to show or hide the player output.
I recommend setting up the user service with systemd and start it with dbus activation. The config files for this are included in conf.

#### info

This is responsible for formatting the raw data into a format that can be understood by a bar. 

##### polybar

Prints a format string that should "just work" as custom module.

##### yambar

Outputs tokens

##### waybar

unlike polybar waybar only allows one on-click handler for the entire element. because of this the output is split into 6 different elements, that can then each be included as custom module
expects 6 sockets from systemd. Writes output for the different parts to each of those streams.

#### info-wasybar-cat

opens one of the sockets and writes to stdout.

#### toggle-hide

toggles the hide status
