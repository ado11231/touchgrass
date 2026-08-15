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

`touchgrass` on Sat Nov 7, after a year of use:

```
  ▀█▀ █▀█ █ █ █▀▀ █ █   █▀▀ █▀█ ▄▀█ █▀ █▀
   █  █▄█ █▄█ █▄▄ █▀█   █▄█ █▀▄ █▀█ ▄█ ▄█

         Nov   Dec       Jan     Feb     Mar       Apr     May     Jun       Jul     Aug       Sep     Oct     Nov
  Mon    ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■
         ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■
  Wed    ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■
         ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■
  Fri    ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■
         ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■
         ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■

         less ■ ■ ■ ■ ■ more

  Today          4h 12m
  This week     18h 40m
  All time    1284h 30m

  Today by tool
  Claude Code  ████████░░░░░░░░  2h 01m
  VS Code      █████░░░░░░░░░░░  1h 15m
  Terminal     ███░░░░░░░░░░░░░    41m
  Codex        █░░░░░░░░░░░░░░░    15m
```

- **Title:** big block letters in bright light magenta, like the Hermes Agent CLI
- **Heatmap:** the last 52 weeks, one square per day. It starts at your install week and fills in over time.
- **Stats:** totals, then today's time per tool, most used first. Bars are each tool's share of today.

Square color by total time that day:

| Time | Color |
| --- | --- |
| None | `#2b2b2b` dark gray |
| Under 1h | `#3d1a4d` deep purple |
| 1h to 3h | `#6b2a85` dark magenta |
| 3h to 6h | `#a53fcc` magenta |
| 6h or more | `#e070ff` bright magenta |

Bars are bright magenta `█` on dark gray `░`.

### Year view

`touchgrass 2026`:

```
  ▀█▀ █▀█ █ █ █▀▀ █ █   █▀▀ █▀█ ▄▀█ █▀ █▀
   █  █▄█ █▄█ █▄▄ █▀█   █▄█ █▀▄ █▀█ ▄█ ▄█

         Jan       Feb     Mar       Apr     May     Jun       Jul     Aug       Sep     Oct     Nov       Dec
  Mon      ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■
           ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■
  Wed      ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■
         ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■
  Fri    ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■
         ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■
         ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■ ■

         less ■ ■ ■ ■ ■ more

  2026 total  312h 40m

  2026 by tool
  Claude Code  ███████░░░░░░░░░  140h 10m
  VS Code      █████░░░░░░░░░░░   98h 25m
  Terminal     ███░░░░░░░░░░░░░   52h 30m
  Codex        █░░░░░░░░░░░░░░░   21h 35m
```

- Every day of the year shows. Days before install or in the future are empty gray.
- Bars are each tool's share of the year
- Narrow terminal? Prints Jan–Jun, then Jul–Dec.

### Rules

- Nothing resets at New Year. The default view just keeps sliding.
- Old weeks slide off the heatmap, but their time stays in all time
- Color turns off with `NO_COLOR` or when output isn't a terminal. Squares become `· ░ ▒ ▓ █`.
- Without full color, uses the closest 256-color shades
