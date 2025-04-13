## Overview

- Uses WAL mode
- Stream WAL to FDB, read SQLite pages mapped to 

## Pragmas

TODO: caches, etc

## Pages

- Page size is 8 KiB which is less than the FDB 10 KB so it's efficient
- Larger pages = less round trips to the database

## Shared Memory File

https://sqlite.org/tempfiles.html#shared_memory_files

> In fact, if there were a mechanism by which SQLite could tell the operating system to never persist the shm file to disk but always hold it in cache memory, SQLite would use that mechanism to avoid any unnecessary disk I/O associated with the shm file. However, no such mechanism exists in standard posix.
>
> https://sqlite.org/walformat.html#the_wal_index_or_shm_file

- We implement SHM to use an in-memory buffer instead of the file system to reduce filesystem operations
- SQLite never fsync anyways but whatever

Future work:

- Potentiall mount the file in a RAM disk instead of implementing this ourselves since it'll be slower

## WAL

Format: https://sqlite.org/walformat.html

Writing to WAL:

- We have a incremental WAL parser
- We assume that SQLite is always writing to the end of the WAL
- We feed those bytes to the WAL parser and it spits out the frames

Reading from WAL:

- We convert the byte index to the appropriate entry with this formula: TODO
- We read the entry and convert it back to bytes that SQLite can understand

## Things to check

- Fix non-exhaustive file types
- Update WAL parser to be stateless
- Are we doing unnecesary writes that operate outside of an fsync
- Are we we panicking if the wal vfs does something we don't expect (eg read middle of the file)

## Optimizations

- Compression
- Output gates
- Reduce clones

