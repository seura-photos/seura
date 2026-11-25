# Cloud-only edit files are downloaded when read

2026-11-25

Nevada found that Seura downloads cloud-only edit files. Photos are never downloaded except as LIB-9 says, but tags, ratings, and edits exist only in the `.seura.json` files, so Seura has to read them, and reading a cloud-only file downloads it. Claude Code traced the three cases: a folder's first scan (every edit file in it), a launch after the cache folder was deleted by hand (every edit file in the library, since the index is gone), and making an edited photo's thumbnail after Clear or when it's missing (that photo's edit file only).

Edit files are a few KB, so this is expected rather than a bug, but LIB-9 had said only what downloads photos. The design session wrote the behavior into LIB-9 and the technical design's cloud section, and the user guide now says that deleting the cache folder by hand makes the next launch read every edit file, while Clear in Settings doesn't.
