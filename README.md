# hyprosd

Hyprland-focused Wayland OSD daemon for volume, brightness, Caps Lock, and Num
Lock.

hyprosd runs as a small background daemon and displays a compact GTK layer-shell
overlay when your media, brightness, or lock-state keys change. It is designed
for Hyprland keybinds and Wayland sessions.

## Install

### AUR

```bash
yay -S hyprosd-git
```

or:

```bash
paru -S hyprosd-git
```

The package installs the `hyprosd` binary. It does not start the daemon or edit
your Hyprland config automatically.

### Manual

```bash
git clone https://github.com/jameswylde/hyprosd.git
cd hyprosd
./scripts/install.sh
```

This installs `hyprosd` to `~/.local/bin/hyprosd` and a convenience launcher to
`~/.local/bin/hyprosd-daemon`.

## Hyprland Setup

Start the daemon from `hyprland.conf`:

```ini
exec-once = hyprosd daemon
```

If you installed manually and `~/.local/bin` is not in your session `PATH`, use:

```ini
exec-once = ~/.local/bin/hyprosd-daemon
```

Add keybinds that update the system value and then trigger the OSD:

```ini
# Audio
bind = ,XF86AudioLowerVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 10%- && hyprosd show volume
bind = ,XF86AudioRaiseVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 10%+ && hyprosd show volume
bind = ,XF86AudioMute, exec, wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle && hyprosd show volume

# Brightness
bind = ,XF86MonBrightnessDown, exec, brightnessctl s 10%- && hyprosd show brightness
bind = ,XF86MonBrightnessUp, exec, brightnessctl s +10% && hyprosd show brightness
```

Restart Hyprland, reload your config, or start the daemon manually once:

```bash
hyprosd daemon
```

## Commands

```bash
hyprosd daemon
hyprosd show volume
hyprosd show brightness
hyprosd show caps
hyprosd show num
```

You can also pass explicit values:

```bash
hyprosd show volume 50
hyprosd show volume 0 --muted
hyprosd show brightness 80
hyprosd show caps on
hyprosd show num off
```

## Optional Tools

For full functionality, install:

- `wireplumber` for `wpctl` volume detection
- `pulseaudio-utils` for fallback `pactl` volume detection
- `brightnessctl` for the example brightness keybinds
- `hyprland`

## Layer Rules

hyprosd sets its layer-shell namespace to `hyprosd`, so it appears in:

```bash
hyprctl layers
```

You can target it with Hyprland layer rules:

```ini
layerrule = blur, hyprosd
layerrule = ignorealpha 0.1, hyprosd
```

## Uninstall

For AUR installs:

```bash
sudo pacman -R hyprosd-git
```

For manual installs:

```bash
./scripts/uninstall.sh
```
