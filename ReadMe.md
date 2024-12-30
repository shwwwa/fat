# File Analysis Tool
File Analysis Tool, fat-rs for short, is a tool to analyze metadata of files, tool to guess file's extension and what it is for.
Maybe later there will be support for conversion and viewing.
## What it supports
Right now it only supports zip/rar archives (it still can provide general info about other file formats though)
## Roadmap
Building CLI, then GUI.
Support of modules, extensive library of all extensions that exist in this world (even if nobody uses them, kind of save-history project)
## Other sources
https://github.com/mmalecot/file-format - more generic scope on project (more useful one if you're trying to make a SPECIFIC reader for project, not general one like mine)
a lot of code borrowed from here to determine extensions
https://www.iana.org/ - mime types
http://fileformats.archiveteam.org/ - contains a lot of useful info about file formats and their identification