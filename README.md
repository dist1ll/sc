# `sc` - Shared Calendar Viewer

This software lets you view your shared calendars
directly in the terminal. 

#### Output

```
[usr@pc]$ sc -d 3

 ┌2022-10-02────────┐┌2022-10-03────────┐ ┌2022-10-04─────────┐ 
 │Find Nemo         ││Nemo Meeting      │ │Meeting with XYZ   │
 │Briefing          ││Agile Coach (skip)│ │Finish Nemo Report │
 │Weekly Meeting    ││                  │ │Order Laptop       │
 │Dentist Appoint.  ││                  │ │                   │
 │Sync with XYZ     ││                  │ │                   │
 └──────────────────┘└──────────────────┘ └───────────────────┘
```

#### Usage 

```
[usr@pc]$ sc --help

Command-line utility for viewing shared calendars

Usage: sc [OPTIONS]
       sc <COMMAND>

Commands:
  add     Add shared calendar by URL
  list    List all calendars
  update  Updates all calendars
  remove  Delete calendar with given ID
  clean   Clean local cache

Options:
  -d, --days <n>  Display events for the next n days
  -t, --today     Display all events for today in detail
  -h, --help      Print help information
  -V, --version   Print version information
```
