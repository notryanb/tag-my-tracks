tag-my-tracks (tmt)
---
tag-my-tracks is a command line tool to manage id3 tags of your music library.

## Why the name?
`tmt` is quick to type when you're already in a terminal. 
None of the songs or artists in my music library seemed to convey "cli tool for managing mp3 tags",
so I just took `tmt` and made up something that fit. ¯\_(ツ)_/¯

## Todo:
- Figure out why quicli verbosity logging isn't working
- Add ability to send in file or directory
- File (make optional and mark as file)
    - View tag info
    - Change tag info with flags
- Directory (make optional and mark as directory)
    - stop recursion and only do top level
    - list all found
    - view tag info for each
    - view frame info for all with flag
    - change frame info for all with flag and confirmation (default is no).