# hyprosd

Simple OSD for hyprland/Wayland, featuring volume, brightness and lock-states.


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

This installs `hyprosd` to `~/.local/bin/hyprosd` and a convenience launcher to
`~/.local/bin/hyprosd-daemon`.

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


## Uninstall

For AUR installs:

```bash
sudo pacman -R hyprosd-git
```

For manual installs:

```bash
./scripts/uninstall.sh
```
