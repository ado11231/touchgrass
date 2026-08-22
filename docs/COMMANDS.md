# Commands

| Command | What it does |
| --- | --- |
| `touchgrass` | Prints your time |
| `touchgrass 2026` | Prints one full year, Jan to Dec |
| `touchgrass track` | Runs the tracker in the terminal |
| `touchgrass install` | Starts the tracker now and on every login |
| `touchgrass uninstall` | Stops the tracker and removes autostart |
| `touchgrass help` | Lists commands |

## Notes

- `touchgrass` works even if the tracker isn't running
- `track` exits if another tracker is already running. Ctrl+C stops it and saves first.
- `install` and `uninstall` are safe to run twice
- `uninstall` keeps your saved time and config

## Output

`touchgrass` on Wed Nov 4, after a year of use:

```
  ████████╗   ██████╗   ██╗   ██╗   ██████╗  ██╗  ██╗   ██████╗   ██████╗    █████╗   ███████╗  ███████╗
  ╚══██╔══╝  ██╔═══██╗  ██║   ██║  ██╔════╝  ██║  ██║  ██╔════╝   ██╔══██╗  ██╔══██╗  ██╔════╝  ██╔════╝
     ██║     ██║   ██║  ██║   ██║  ██║       ███████║  ██║  ███╗  ██████╔╝  ███████║  ███████╗  ███████╗
     ██║     ██║   ██║  ██║   ██║  ██║       ██╔══██║  ██║   ██║  ██╔══██╗  ██╔══██║  ╚════██║  ╚════██║
     ██║     ╚██████╔╝  ╚██████╔╝  ╚██████╗  ██║  ██║  ╚██████╔╝  ██║  ██║  ██║  ██║  ███████║  ███████║
     ╚═╝      ╚═════╝    ╚═════╝    ╚═════╝  ╚═╝  ╚═╝   ╚═════╝   ╚═╝  ╚═╝  ╚═╝  ╚═╝  ╚══════╝  ╚══════╝

  ╭─────────────────────────────────────────────────────────────────────────────────────────────────────────────╮
  │     Nov   Dec     Jan     Feb     Mar       Apr     May       Jun     Jul     Aug       Sep     Oct     Nov │
  │ Mon 🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓│
  │     🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓│
  │ Wed 🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓│
  │     🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓  │
  │ Fri 🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓  │
  │     🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓  │
  │     🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓  │
  │                                                                                                             │
  ╰─────────────────────────────────────────────────────────────────────────────────────────────────────────────╯
                                                                                              less 🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓 more

  TODAY             THIS WEEK         ALL TIME
  4h 12m            11h 12m           686h 42m

  Claude Code  ████████████░░░░░░░░░░░░     2h 01m   48%
  Terminal     █████████░░░░░░░░░░░░░░░     1h 30m   36%
  VS Code      ███████░░░░░░░░░░░░░░░░░     1h 15m   30%
  Codex        █░░░░░░░░░░░░░░░░░░░░░░░     0h 15m    6%
```

- **Title:** large shadowed letters, like the LazyVim logo. Faces are bright magenta, shadows dark magenta. Letters spread out on wide terminals. Narrow terminals get the small two-line title.
- **Heatmap:** the last 52 weeks in a white rounded border, one square per day, drawn with sextant blocks (1.5 cells wide, half-cell gaps). The full grid shows from day one, like GitHub. Labels, border, and squares are one space apart.
- **Stats:** today, this week, and all time side by side, then today's time per tool with no heading, most used first. Bars and percentages are each tool's share of today's active time. Tools overlap (Claude Code inside kitty), so percentages can add up to more than 100%.

Square color by total time that day:

| Time | Color |
| --- | --- |
| None | `#2b2b2b` dark gray |
| Under 1h | `#522468` deep purple |
| 1h to 3h | `#6b2a85` dark magenta |
| 3h to 6h | `#a53fcc` magenta |
| 6h or more | `#e070ff` bright magenta |

Bars are bright magenta `█` on dark gray `░`. Labels, the legend, and the heatmap border are white.

### Year view

`touchgrass 2026`:

```
  ████████╗   ██████╗   ██╗   ██╗   ██████╗  ██╗  ██╗   ██████╗   ██████╗    █████╗   ███████╗  ███████╗
  ╚══██╔══╝  ██╔═══██╗  ██║   ██║  ██╔════╝  ██║  ██║  ██╔════╝   ██╔══██╗  ██╔══██╗  ██╔════╝  ██╔════╝
     ██║     ██║   ██║  ██║   ██║  ██║       ███████║  ██║  ███╗  ██████╔╝  ███████║  ███████╗  ███████╗
     ██║     ██║   ██║  ██║   ██║  ██║       ██╔══██║  ██║   ██║  ██╔══██╗  ██╔══██║  ╚════██║  ╚════██║
     ██║     ╚██████╔╝  ╚██████╔╝  ╚██████╗  ██║  ██║  ╚██████╔╝  ██║  ██║  ██║  ██║  ███████║  ███████║
     ╚═╝      ╚═════╝    ╚═════╝    ╚═════╝  ╚═╝  ╚═╝   ╚═════╝   ╚═╝  ╚═╝  ╚═╝  ╚═╝  ╚══════╝  ╚══════╝

  ╭───────────────────────────────────────────────────────────────────────────────────────────────────────────────╮
  │     Jan     Feb     Mar       Apr     May       Jun     Jul     Aug       Sep     Oct     Nov       Dec       │
  │ Mon   🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓│
  │       🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓│
  │ Wed   🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓│
  │     🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓│
  │ Fri 🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓  │
  │     🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓  │
  │     🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓  │
  │                                                                                                               │
  ╰───────────────────────────────────────────────────────────────────────────────────────────────────────────────╯
                                                                                                less 🬹🬓🬹🬓🬹🬓🬹🬓🬹🬓 more

  2026 TOTAL        ACTIVE DAYS       BEST DAY
  562h 12m          203               Jan 9  7h 00m

  Claude Code  █████████████░░░░░░░░░░░   308h 55m   55%
  Terminal     ██████████░░░░░░░░░░░░░░   224h 42m   40%
  VS Code      ████████░░░░░░░░░░░░░░░░   196h 29m   35%
  Codex        ██░░░░░░░░░░░░░░░░░░░░░░    54h 00m   10%
```

- Every day of the year shows. Days before install or in the future are empty gray.
- Summary shows the year total, active days, and best day, then each tool's time for the year
- Bars are each tool's share of the year's active time
- Narrow terminal? Prints Jan-Jun, then Jul-Dec. Very narrow windows split those ranges into smaller panels.

### Rules

- Nothing resets at New Year. The default view just keeps sliding.
- Old weeks slide off the heatmap, but their time stays in all time
- The current week is one of the 52 columns. Narrow default views show fewer recent weeks.
- Color turns off with `NO_COLOR` or when output isn't a terminal. Squares become `· ░ ▒ ▓ █`.
- Without full color, uses the closest 256-color shades
