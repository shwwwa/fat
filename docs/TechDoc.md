There are still a lot of issues with the concept.

# Introduction
Fat is the ***file analysis tool***. It is divided into following 7 subcommands:

- [ ] fat recognize - recognizes file's extension by analyzing it
- [ ] fat analyze - analyzes files for strange things in it, extracts any data from it that does not belong in it, searches for sfx, encryption..
- [ ] fat test - tests for file errors (like broken header in archive)
- [ ] fat general - provides general properties for file 
- [ ] fat metadata - extracts metadata from file (exif from photos, videos, file changed from archive etc..), does not extract data.
- [ ] fat data - provides data contained at file, extracts it on demand
- [ ] fat check - check for dependencies found in fat (it will eventually depend on somtehing!)

Helper subcommands (may not be implemented):
- [x] fat help = fat --help but later about submodules
- [x] fat version = provides version of dependencies?
- [ ] fat list - list of deps?
- [ ] fat update - looks if update is available (but installs only for deps)
- [ ] fat install - install dependencies
- [ ] fat remove - remove dependencies
- [ ] fat scan - scan for files in directory/directories
- [ ] fat diff - compares two files (maybe more in the future)

# Technical info (contributing info)
- Subcommands of subcommands are not allowed to not confuse users. Use options instead.
