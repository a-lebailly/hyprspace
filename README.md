# hyprspace

A clean and powerful **workspace launcher & layout generator** for Hyprland. Hyprspace lets you organize your custom workspace environments as simple scripts, browse them in an intuitive terminal UI, and launch them instantly.

Instead of manually managing scattered shell scripts or rewriting layouts each time, **hyprspace** centralizes everything under one directory and provides:

* A **TUI selector** for your workspace presets
* An **interactive wizard** to create new layouts
* Automatic detection of the workspace targeted by each script
* One‑command launching of structured layouts powered by `hyprctl`

Hyprspace is especially useful if you maintain several dashboards, productivity setups, or complex floating arrangements that you want to summon on demand.

---

## Installation

### Automatic install (recommended)

```bash
curl -sSL https://raw.githubusercontent.com/a-lebailly/hyprspace/main/install.sh | bash
```

This downloads the latest prebuilt binary into the current directory.
To install it system‑wide:

```bash
sudo mv ./hyprspace /usr/local/bin/hyprspace
```

### Build from source

**Requirements:**

* Rust

```bash
git clone https://github.com/a-lebailly/hyprspace.git
cd hyprspace
chmod +x build.sh
./build.sh
```

The optimized release binary will be available at:

```bash
dist/hyprspace
```

Install globally:

```bash
sudo mv dist/hyprspace /usr/local/bin/hyprspace
```

---

## Usage

Launch the TUI:

```bash
hyprspace
```

Inside the interface:

* Navigate with `↑/↓` or `j/k`
* Press `Enter` to launch a workspace or create a new one
* Press `q` or `Esc` to quit

### Optional: Add a Hyprland keybinding

You can add a shortcut to launch hyprspace directly from Hyprland (~/.config/hypr/hyprland.conf).  
**Example**: launch hyprspace with `$mainMod + SHIFT + n` in a centered floating window.

Add this to your Hyprland config:
```
bind = $mainMod SHIFT, n, exec, kitty --title hyprspace-selector hyprspace
```

To center and size the selector window:
```
windowrulev2 = float, title:^(hyprspace-selector)$
windowrulev2 = center, title:^(hyprspace-selector)$
windowrulev2 = size 15% 50%, title:^(hyprspace-selector)$
```

---

## Creating workspace layouts

Hyprspace includes an **interactive wizard** that guides you through the creation of a new layout script.

You will be asked to:

1. Choose a **workspace number** (e.g., 1, 3, 5…)
2. Choose a **script name** (used to create `workspace-name.sh`)
3. Add any number of **window rules**:

   * Size: width / height (e.g., `50%`, `30%`)
   * Position: X / Y (e.g., `5%`, `10%`)
   * Command to launch (e.g., `kitty`, `firefox`, `thunar`)

Hyprspace automatically generates:

* A workspace switch (`hyprctl dispatch workspace N`)
* A helper `rule_exec` function
* Structured window layout commands
* An executable script saved under:

```
~/.config/hyprspace/workspace-<name>.sh
```

---

## Script format

Generated scripts follow this structure:

```bash
#!/bin/bash

hyprctl dispatch workspace 4

rule_exec() {
  local rules="$1"
  shift
  hyprctl dispatch exec "[$rules] $*"
}

# Example window
done
```

Hyprspace parses the script to detect the workspace number but leaves the layout logic entirely in your hands.

---

## Notes

* Scripts are stored in `~/.config/hyprspace`
* Filenames must follow: `workspace-name.sh`
* Hyprspace will never overwrite existing scripts
* Works perfectly with **floating**, **tiled**, or mixed setups