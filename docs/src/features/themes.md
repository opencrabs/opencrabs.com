# Themes

The TUI is colored by **semantic roles**, not raw hexes. Every render site resolves a role
(`accent`, `text_primary`, `selection_bg`, …) through the active theme, so one switch restyles
the whole interface at runtime. A theme is **one file, 43 keys, done**.

## Switching themes

| Command | What it does |
|---|---|
| `/theme` | Opens the interactive picker: `↑`/`↓` (or `j`/`k`) move the cursor and **preview live**, the whole UI recolors under the arrow keys. `Enter` applies and persists, `Esc` (or `q`) cancels and reverts to whatever was active when the picker opened. |
| `/theme list` | Lists built-in themes and user presets, then re-scans `~/.opencrabs/themes/`. Files that fail validation are listed with a human-readable rejection reason, never silently ignored. |
| `/theme set <name>` | Applies a theme immediately and writes it to your config. Lookup is case-insensitive across built-ins and user presets; an unknown name tells you to run `/theme list`. |
| `/theme reset` | Returns to the default (`crab-dark`) and clears the config key. |

To pin a theme without the picker, set it in `config.toml`:

```toml
[tui]
theme = "nord"
```

The value is applied at boot before the first frame. An empty or unknown name falls back to
`crab-dark`. User theme files are re-scanned on every `/theme` invocation, so editing a preset
file hot-loads it without restarting the TUI.

## Built-in themes

There are **39** built-in themes: 8 hand-tuned presets plus a curated pack of 31 converted
from popular upstream palettes (see [Imported themes](#imported-themes)). `crab-dark` is the
default and matches the historical OpenCrabs look.

| Preset | Vibe |
|---|---|
| `crab-dark` | The house palette: burnt-orange accent, cool grays, deep ink |
| `dracula` | The classic purple-blue dark theme with orange accent |
| `alucard` | Warm dark, muted contrast (Castlevania's good-boy Dracula) |
| `monokai` | Sublime's iconic warm dark with pink/yellow |
| `catppuccin-mocha` | Pastel dark, soft and low-contrast |
| `catppuccin-latte` | The same pastels on light surfaces |
| `solarized-light` | Ethan Schoonlif's light base |
| `solarized-dark` | The original dark base |

The curated pack adds: `aura`, `ayu`, `carbonfox`, `catppuccin-frappe`, `catppuccin-macchiato`,
`cobalt2`, `cursor`, `everforest`, `flexoki`, `github`, `gruvbox`, `gruvbox-light`, `kanagawa`,
`material`, `matrix`, `mercury`, `nightowl`, `nord`, `one-dark`, `opencode`, `orng`,
`osaka-jade`, `palenight`, `rosepine`, `rosepine-dawn`, `synthwave84`, `tokyonight`,
`tokyonight-storm`, `vercel`, `vesper`, `zenburn`.

## Gallery

Swatches show four anchor roles per theme (accent, body text, success, ink background), read
from the shipped sources. Regenerate with `python3 docs/scripts/gen_theme_gallery.py`.

<!-- GALLERY:BEGIN -->
| Theme | Type | accent | text | success | ink | Source |
|---|---|---|---|---|---|---|
| **crab-dark** | default | <span style="background:#d76414;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#d76414` | <span style="background:#c8c8d2;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#c8c8d2` | <span style="background:#50c878;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#50c878` | <span style="background:#14141e;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#14141e` | built-in |
| **dracula** | hand-built | <span style="background:#ffb86c;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#ffb86c` | <span style="background:#f8f8f2;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#f8f8f2` | <span style="background:#50fa7b;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#50fa7b` | <span style="background:#282a36;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#282a36` | built-in |
| **alucard** | hand-built | <span style="background:#a34d14;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#a34d14` | <span style="background:#1f1f1f;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#1f1f1f` | <span style="background:#14710a;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#14710a` | <span style="background:#1f1f1f;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#1f1f1f` | built-in |
| **monokai** | hand-built | <span style="background:#fd971f;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#fd971f` | <span style="background:#f8f8f2;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#f8f8f2` | <span style="background:#a6e22e;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#a6e22e` | <span style="background:#272822;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#272822` | built-in |
| **catppuccin-mocha** | hand-built | <span style="background:#fab387;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#fab387` | <span style="background:#cdd6f4;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#cdd6f4` | <span style="background:#a6e3a1;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#a6e3a1` | <span style="background:#1e1e2e;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#1e1e2e` | built-in |
| **catppuccin-latte** | hand-built | <span style="background:#fe640b;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#fe640b` | <span style="background:#4c4f69;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#4c4f69` | <span style="background:#40a02b;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#40a02b` | <span style="background:#4c4f69;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#4c4f69` | built-in |
| **solarized-light** | hand-built | <span style="background:#cb4b16;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#cb4b16` | <span style="background:#657b83;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#657b83` | <span style="background:#859900;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#859900` | <span style="background:#073642;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#073642` | built-in |
| **solarized-dark** | hand-built | <span style="background:#cb4b16;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#cb4b16` | <span style="background:#839496;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#839496` | <span style="background:#859900;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#859900` | <span style="background:#002b36;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#002b36` | built-in |
| **aura** | curated pack | <span style="background:#a277ff;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#a277ff` | <span style="background:#edecee;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#edecee` | <span style="background:#61ffca;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#61ffca` | <span style="background:#0f0f0f;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#0f0f0f` | opencode, dark, 2026-09-11 |
| **ayu** | curated pack | <span style="background:#e6b450;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#e6b450` | <span style="background:#bfbdb6;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#bfbdb6` | <span style="background:#7fd962;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#7fd962` | <span style="background:#0b0e14;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#0b0e14` | opencode, dark, 2026-09-11 |
| **carbonfox** | curated pack | <span style="background:#ff7eb6;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#ff7eb6` | <span style="background:#f2f4f8;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#f2f4f8` | <span style="background:#25be6a;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#25be6a` | <span style="background:#161616;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#161616` | opencode, dark, 2026-09-11 |
| **catppuccin-macchiato** | curated pack | <span style="background:#f5bde6;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#f5bde6` | <span style="background:#cad3f5;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#cad3f5` | <span style="background:#a6da95;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#a6da95` | <span style="background:#24273a;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#24273a` | opencode, dark, 2026-09-11 |
| **cobalt2** | curated pack | <span style="background:#2affdf;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#2affdf` | <span style="background:#ffffff;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#ffffff` | <span style="background:#9eff80;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#9eff80` | <span style="background:#193549;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#193549` | opencode, dark, 2026-09-11 |
| **cursor** | curated pack | <span style="background:#88c0d0;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#88c0d0` | <span style="background:#e4e4e4;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#e4e4e4` | <span style="background:#3fa266;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#3fa266` | <span style="background:#181818;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#181818` | opencode, dark, 2026-09-11 |
| **everforest** | curated pack | <span style="background:#d699b6;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#d699b6` | <span style="background:#d3c6aa;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#d3c6aa` | <span style="background:#a7c080;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#a7c080` | <span style="background:#2d353b;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#2d353b` | opencode, dark, 2026-09-11 |
| **flexoki** | curated pack | <span style="background:#8b7ec8;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#8b7ec8` | <span style="background:#cecdc3;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#cecdc3` | <span style="background:#879a39;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#879a39` | <span style="background:#100f0f;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#100f0f` | opencode, dark, 2026-09-11 |
| **github** | curated pack | <span style="background:#39c5cf;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#39c5cf` | <span style="background:#c9d1d9;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#c9d1d9` | <span style="background:#3fb950;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#3fb950` | <span style="background:#0d1117;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#0d1117` | opencode, dark, 2026-09-11 |
| **gruvbox** | curated pack | <span style="background:#8ec07c;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#8ec07c` | <span style="background:#ebdbb2;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#ebdbb2` | <span style="background:#b8bb26;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#b8bb26` | <span style="background:#282828;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#282828` | opencode, dark, 2026-09-11 |
| **kanagawa** | curated pack | <span style="background:#d27e99;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#d27e99` | <span style="background:#dcd7ba;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#dcd7ba` | <span style="background:#98bb6c;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#98bb6c` | <span style="background:#1f1f28;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#1f1f28` | opencode, dark, 2026-09-11 |
| **material** | curated pack | <span style="background:#89ddff;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#89ddff` | <span style="background:#eeffff;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#eeffff` | <span style="background:#c3e88d;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#c3e88d` | <span style="background:#263238;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#263238` | opencode, dark, 2026-09-11 |
| **matrix** | curated pack | <span style="background:#c770ff;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#c770ff` | <span style="background:#62ff94;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#62ff94` | <span style="background:#62ff94;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#62ff94` | <span style="background:#0a0e0a;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#0a0e0a` | opencode, dark, 2026-09-11 |
| **mercury** | curated pack | <span style="background:#8da4f5;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#8da4f5` | <span style="background:#dddde5;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#dddde5` | <span style="background:#77c599;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#77c599` | <span style="background:#171721;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#171721` | opencode, dark, 2026-09-11 |
| **nightowl** | curated pack | <span style="background:#c792ea;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#c792ea` | <span style="background:#d6deeb;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#d6deeb` | <span style="background:#c5e478;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#c5e478` | <span style="background:#011627;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#011627` | opencode, dark, 2026-09-11 |
| **nord** | curated pack | <span style="background:#8fbcbb;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#8fbcbb` | <span style="background:#eceff4;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#eceff4` | <span style="background:#a3be8c;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#a3be8c` | <span style="background:#2e3440;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#2e3440` | opencode, dark, 2026-09-11 |
| **one-dark** | curated pack | <span style="background:#56b6c2;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#56b6c2` | <span style="background:#abb2bf;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#abb2bf` | <span style="background:#98c379;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#98c379` | <span style="background:#282c34;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#282c34` | opencode, dark, 2026-09-11 |
| **opencode** | curated pack | <span style="background:#9d7cd8;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#9d7cd8` | <span style="background:#eeeeee;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#eeeeee` | <span style="background:#7fd88f;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#7fd88f` | <span style="background:#0a0a0a;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#0a0a0a` | opencode, dark, 2026-09-11 |
| **orng** | curated pack | <span style="background:#fff7f1;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#fff7f1` | <span style="background:#eeeeee;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#eeeeee` | <span style="background:#6ba1e6;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#6ba1e6` | <span style="background:#0a0a0a;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#0a0a0a` | opencode, dark, 2026-09-11 |
| **osaka-jade** | curated pack | <span style="background:#549e6a;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#549e6a` | <span style="background:#c1c497;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#c1c497` | <span style="background:#549e6a;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#549e6a` | <span style="background:#111c18;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#111c18` | opencode, dark, 2026-09-11 |
| **palenight** | curated pack | <span style="background:#89ddff;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#89ddff` | <span style="background:#a6accd;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#a6accd` | <span style="background:#c3e88d;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#c3e88d` | <span style="background:#292d3e;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#292d3e` | opencode, dark, 2026-09-11 |
| **rosepine** | curated pack | <span style="background:#ebbcba;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#ebbcba` | <span style="background:#e0def4;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#e0def4` | <span style="background:#31748f;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#31748f` | <span style="background:#191724;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#191724` | opencode, dark, 2026-09-11 |
| **synthwave84** | curated pack | <span style="background:#b084eb;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#b084eb` | <span style="background:#ffffff;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#ffffff` | <span style="background:#72f1b8;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#72f1b8` | <span style="background:#262335;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#262335` | opencode, dark, 2026-09-11 |
| **tokyonight** | curated pack | <span style="background:#ff966c;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#ff966c` | <span style="background:#c8d3f5;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#c8d3f5` | <span style="background:#c3e88d;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#c3e88d` | <span style="background:#1a1b26;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#1a1b26` | opencode, dark, 2026-09-11 |
| **vercel** | curated pack | <span style="background:#8e4ec6;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#8e4ec6` | <span style="background:#ededed;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#ededed` | <span style="background:#46a758;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#46a758` | <span style="background:#000000;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#000000` | opencode, dark, 2026-09-11 |
| **vesper** | curated pack | <span style="background:#ffc799;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#ffc799` | <span style="background:#ffffff;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#ffffff` | <span style="background:#99ffe4;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#99ffe4` | <span style="background:#101010;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#101010` | opencode, dark, 2026-09-11 |
| **zenburn** | curated pack | <span style="background:#93e0e3;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#93e0e3` | <span style="background:#dcdccc;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#dcdccc` | <span style="background:#7f9f7f;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#7f9f7f` | <span style="background:#3f3f3f;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#3f3f3f` | opencode, dark, 2026-09-11 |
| **catppuccin-frappe** | curated pack | <span style="background:#e5c890;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#e5c890` | <span style="background:#c6d0f5;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#c6d0f5` | <span style="background:#a6d189;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#a6d189` | <span style="background:#303446;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#303446` | alacritty-theme, 2026-09-11 |
| **gruvbox-light** | curated pack | <span style="background:#b57614;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#b57614` | <span style="background:#3c3836;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#3c3836` | <span style="background:#98971a;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#98971a` | <span style="background:#fbf1c7;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#fbf1c7` | alacritty-theme, 2026-09-11 |
| **rosepine-dawn** | curated pack | <span style="background:#d38d2f;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#d38d2f` | <span style="background:#575279;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#575279` | <span style="background:#286983;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#286983` | <span style="background:#faf4ed;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#faf4ed` | alacritty-theme, 2026-09-11 |
| **tokyonight-storm** | curated pack | <span style="background:#ff9e64;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#ff9e64` | <span style="background:#a9b1d6;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#a9b1d6` | <span style="background:#9ece6a;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#9ece6a` | <span style="background:#24283b;width:.9em;height:.9em;display:inline-block;border-radius:2px"></span> `#24283b` | alacritty-theme, 2026-09-11 |
<!-- GALLERY:END -->

## The 43 roles

A user theme is a flat TOML file mapping every role to a `#RRGGBB` hex. Role names double as
the TOML keys. What each one colors:

| Group | Key | Colors |
|---|---|---|
| Accents | `accent` | Primary emphasis: prompts, active states, the crab orange |
| | `accent_teal` | Primary action / selected accent |
| | `accent_soft` | Passive, informational accent (soft white) |
| Text | `text_primary` | Body text |
| | `text_secondary` | Secondary text |
| | `text_muted` | Muted labels |
| | `text_dim` | Deepest text tier |
| Chrome grays | `gray` | Borders, passive labels, info lines (the workhorse) |
| | `gray_mid` | Secondary borders, quiet panel text |
| | `gray_detail` | Detail rows under primary rows (tool list) |
| | `gray_dim` | Inactive borders (unfocused input frame) |
| | `gray_dark` | Deeper inactive chrome |
| | `gray_base` | Darkest neutral foreground (plan-widget borders) |
| | `gray_light` | Bright neutral text |
| | `gray_soft` | Soft label text |
| | `gray_muted` | Muted body text (plan widget) |
| Positive | `success` | "ok" states, active pane borders |
| | `analytics_green` | Mission-control bars and values |
| | `green_check` | Checked-markdown checkboxes |
| Negative | `error` | Primary failures |
| | `error_soft` | Error titles |
| | `error_faded` | Error bodies |
| Warning | `warning` | Amber input hints |
| | `warning_muted` | Muted amber |
| | `amber_muted` | Proposed-skill inbox badges |
| Teal tiers | `teal_vivid` | Selection/active in input + tools |
| | `teal_bright` | Dialogs |
| | `teal_muted` | Completed/skipped plan tasks |
| | `teal_calm` | Plan progress bars |
| Blue tiers | `blue_slate` | Help text, input placeholders |
| | `blue_steel` | Informational accents (sessions, files, projects) |
| | `blue_link` | Markdown links |
| | `blue_sky` | Projects |
| | `blue_soft` | Chat accents |
| | `blue_vivid` | Onboarding voice brand accent |
| | `blue_code` | Inline code (markdown) |
| Surfaces | `selection_bg` | Selected text in the composer |
| | `surface_panel` | Mission-control panel background |
| | `surface_qr` | QR panel background (onboarding) |
| | `surface_code` | Code-block background |
| | `surface_code_alt` | Code-block background, alternate shade |
| Ink | `ink` | The main background canvas |
| Misc | `purple_soft` | Proposed-brain-dedup inbox badges |

Decorative animation cycles (like the rotating project badge colors) are deliberately **not**
roles: they stay preset-agnostic.

## Writing your own theme

Drop a file at `~/.opencrabs/themes/<name>.toml`. The file stem **is** the theme name. All 43
keys are required; here is a complete valid theme (`nord`, from the curated pack):

```toml
accent = "#8fbcbb"
accent_teal = "#8fbcbb"
accent_soft = "#eceff4"
text_primary = "#eceff4"
text_secondary = "#8b95a7"
text_muted = "#767b84"
text_dim = "#9ca0a8"
gray = "#434c5e"
gray_mid = "#9ca0a8"
gray_detail = "#91959e"
gray_dim = "#7e838c"
gray_dark = "#6b707a"
gray_base = "#434c5e"
gray_light = "#d0d3d9"
gray_soft = "#afb3ba"
gray_muted = "#8d929a"
success = "#a3be8c"
analytics_green = "#a3be8c"
green_check = "#a3be8c"
error = "#bf616a"
error_soft = "#bf616a"
error_faded = "#6f4853"
warning = "#d08770"
warning_muted = "#9f6e62"
amber_muted = "#87625a"
teal_vivid = "#8fbcbb"
teal_bright = "#8fbcbb"
teal_muted = "#6d8c90"
teal_calm = "#5a7177"
blue_slate = "#698f9e"
blue_steel = "#88c0d0"
blue_link = "#81a1c1"
blue_sky = "#88c0d0"
blue_soft = "#648896"
blue_vivid = "#88c0d0"
blue_code = "#88c0d0"
selection_bg = "#505660"
surface_panel = "#3b4252"
surface_qr = "#343a45"
surface_code = "#434c5e"
surface_code_alt = "#494e59"
ink = "#2e3440"
purple_soft = "#81a1c1"
```

### Validation rules

Validation is loud, never silent. A theme file is rejected, with the reason shown by
`/theme list`, when it:

1. **misses any of the 43 roles**: the error names the missing key;
2. **carries an unknown key**: strict typo guard, there is no partial-schema mode;
3. **uses a malformed value**: only `#RRGGBB` or `RRGGBB`, case-insensitive;
4. **fails the contrast floor** (eight reading-critical (foreground, background) pairs must
   hit **2.5:1** WCAG contrast: `text_primary`/`text_secondary`/`accent`/`success`/`error`/
   `warning` on `ink`, `blue_code` on `surface_code`, and `gray_mid` on `surface_panel`.
   Deliberately dim tiers (`text_muted`, `text_dim`, …) are excluded) dim is their job;
5. **collides with a built-in name**: user files can never shadow a shipped preset.

Everything else is free for the runtime: each valid theme gets its **ANSI-256 fallback tier
derived automatically**, so terminals without truecolor still render it (nearest index in the
6×6×6 cube, grayscale ramp for neutrals).

## Theme enforcement (v0.5.3)

Every widget in the TUI now resolves its colours through the theme engine, the last
hardcoded ANSI sites are gone, and a suite-level guard fails the build if a new raw ANSI
colour site appears (#1634). Themes can also declare an optional **canvas background**
(the source background carries through catalog conversion), so a theme can restyle the
whole terminal canvas, not just the widgets on it.

## Imported themes

The 31-theme curated pack was generated by the in-tree converter (`tui::theme_catalog`) from
two upstream formats:

- **alacritty-theme TOML** (`[colors.primary]` background/foreground plus the 16 ANSI colors): 
  the `alacritty/alacritty-theme` catalog, 176 themes;
- **opencode theme JSON** (`defs` + `theme` token maps with dark/light variants, hex strings,
  ANSI integers, `defs` references and `"none"`), the `sst/opencode` built-in themes,
  33 assets.

Both convert through a **fixed 43-role mapping** (semantic token first, ANSI color or
deterministic bg/fg mix as fallback), and every generated file is pre-certified by the same
validator user themes face: a converted file that doesn't validate never ships. Each pack
file carries a `# source:` provenance header naming the upstream repo, file, and fetch date;
the gallery table shows it. Upstream palettes belong to their authors; the pack imports their
colors, not their names' goodwill.

The converter is generation-time tooling: the TUI itself only knows the embedded presets and
your drop-in files.
