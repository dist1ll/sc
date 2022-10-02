# `sc` - Shared Calendar Viewer

This software lets you view your shared calendars
directly in the terminal. To use, create a shared calendar link
(.ics) for one of the supported platforms, and do the following:


1. `sc add "<url>"` adds the URL to the list of calendars

2. `sc list` shows the list of added calendars. 

3. `sc update` fetches the remote calendar content

4. After successfull update, `sc` will print upcoming events. 


#### Supported Calendars 

- [x] Proton Calendar
- [ ] Microsoft Outlook
- [ ] Google Calendar

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

#### Limitations

Note that shared calendar endpoints may server stale data. 
[From Proton docs:](https://proton.me/support/share-calendar)

> When you update your calendar, anyone with access to your 
calendar will see the updates you make to it. It may take up 
to eight hours before they see the changes. 
