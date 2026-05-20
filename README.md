# hyprosd

![hyprosd screenshot](assets/example.png)

Simple OSD for hyprland/Wayland, featuring volume, brightness and lock-states.

[View on AUR](https://aur.archlinux.org/packages/hyprosd-git) 

## Install

### AUR

```bash
yay -S hyprosd-git
```

```bash
paru -S hyprosd-git
```


### Manual

```bash
git clone https://github.com/jameswylde/hyprosd.git
cd hyprosd
./scripts/install.sh
```

##  Setup

Start the hyprosd daemon from `hyprland.conf`:

```ini
exec-once = hyprosd daemon
```

If you installed manually and `~/.local/bin` is not in your session `PATH`, use:

```ini
exec-once = ~/.local/bin/hyprosd-daemon
```


Restart Hyprland, reload your config, or start the daemon manually once:

```bash
hyprosd daemon
```

Bind your media keys for volume display:

```ini
bind = ,XF86AudioLowerVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%- && hyprosd show volume
bind = ,XF86AudioRaiseVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+ && hyprosd show volume
bind = ,XF86AudioMute, exec, wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle && hyprosd show volume

bind = ,XF86MonBrightnessDown, exec, brightnessctl s 10%- && hyprosd show brightness
bind = ,XF86MonBrightnessUp, exec, brightnessctl s +10% && hyprosd show brightness
```

For AUR installs, `hyprosd` is installed to `/usr/bin/hyprosd`, so using
`hyprosd show ...` is preferred. For manual installs, use `hyprosd show ...` if
`~/.local/bin` is in your session `PATH`; otherwise use the full path:

```ini
bind = ,XF86AudioLowerVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%- && ~/.local/bin/hyprosd show volume
```

If you enable Hyprland blur for the OSD layer, add the below to your hyprland config so the rounded corners do not reveal the squared layer surface:

```ini
layerrule = blur on, match:namespace hyprosd
layerrule = ignore_alpha 0.1, match:namespace hyprosd
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
