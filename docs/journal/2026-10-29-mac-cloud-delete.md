# Deleting photos in OneDrive on the Mac (#112)

- **The design session's answer:** no Finder Apple Events. On the Mac, deleting downloaded OneDrive photos stays off, and the confirmation says why and offers Show in Finder. Claude Code may spend up to an hour looking for an entitlement or bundle key; otherwise it's retested when Developer ID signing is set up before the beta (M7).
- **What's built:** the Library's delete leaves photos in `~/Library/CloudStorage` as they are, on the Mac only, as it already does for cloud-only photos and drives with no Trash.
  - The confirmation counts them and names the service: "2 photos are in OneDrive, where macOS doesn't let Seura Photos move photos to the Trash, so they're left as they are. You can delete them in Finder."
  - Show in Finder reveals just those photos and closes the confirmation.
  - If they're all the photos, the confirmation has OK and Show in Finder, and no Move to Trash.
- **A wider scope than asked, Claude Code's choice, in the pull request for Nevada to confirm:** OneDrive, Google Drive and Dropbox all keep their folders in `~/Library/CloudStorage` and use the same file provider mechanism, so the rule covers that whole folder rather than OneDrive alone. iCloud Drive, in `~/Library/Mobile Documents`, works differently and hasn't shown the problem, so its photos still go to the Trash.
- **Nevada confirmed the wider scope:** "Applying it to all of the cloud storage folder is fine for now. We will assume that they all operate the same way and err on the side of caution." Dropbox and Google Drive get tested later; Nevada has accounts with both but uses only OneDrive now.
- **The hour of looking for an entitlement** needs the Mac. Nevada asked the Mac session to spend it, together with measuring PRF-6 there for #159.
- **What it found, in about half the hour: nothing to add to Seura's bundle, and #112's premise didn't hold.** The Mac session wrote a small probe app, outside the repository, and generated six gray test images in the OneDrive test folder, with Nevada's approval.
  - Every way of trashing a synced file was refused with the same error, 513: FileManager's trashItem (Seura's call), the same inside a file coordinator, and NSWorkspace's recycle.
  - It was refused for every caller: the probe opened like an app, the probe run from the shell, a Swift script, and the same script in Terminal, run by Nevada.
  - One attempt succeeded, a minute after its image was made. By the next attempts OneDrive had synced the images and changed their permissions to match the earlier test photo. So the success was most likely a file OneDrive hadn't taken in yet, and #112's "Terminal deletes it" probably was too.
  - Moving the file into the Trash folder directly, another app's workaround, is refused at the Trash folder, which macOS protects; it would need Full Disk Access and would lose Put Back.
  - No one reports an entitlement or Info.plist fix. Another project (yazi) reports the same error on OneDrive and Dropbox folders only while OneDrive is running, which its maintainer calls a OneDrive bug.
  - So the recorded decision stands: retest with Developer ID signing at M7, though nothing suggests it will change the result, and #161's confirmation is what users see meanwhile. It also supports covering all of `~/Library/CloudStorage`.
  - Not tried, because each changes Nevada's setup: quitting OneDrive to confirm the yazi report, and a logging profile to read fileproviderd's private messages. Five of the test images are left for Nevada to delete in Finder.
