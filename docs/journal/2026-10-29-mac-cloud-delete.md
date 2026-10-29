# Deleting photos in OneDrive on the Mac (#112)

- **The design session's answer:** no Finder Apple Events. On the Mac, deleting downloaded OneDrive photos stays off, and the confirmation says why and offers Show in Finder. Claude Code may spend up to an hour looking for an entitlement or bundle key; otherwise it's retested when Developer ID signing is set up before the beta (M7).
- **What's built:** the Library's delete leaves photos in `~/Library/CloudStorage` as they are, on the Mac only, as it already does for cloud-only photos and drives with no Trash.
  - The confirmation counts them and names the service: "2 photos are in OneDrive, where macOS doesn't let Seura Photos move photos to the Trash, so they're left as they are. You can delete them in Finder."
  - Show in Finder reveals just those photos and closes the confirmation.
  - If they're all the photos, the confirmation has OK and Show in Finder, and no Move to Trash.
- **A wider scope than asked, Claude Code's choice, in the pull request for Nevada to confirm:** OneDrive, Google Drive and Dropbox all keep their folders in `~/Library/CloudStorage` and use the same file provider mechanism, so the rule covers that whole folder rather than OneDrive alone. iCloud Drive, in `~/Library/Mobile Documents`, works differently and hasn't shown the problem, so its photos still go to the Trash.
- **The hour of looking for an entitlement** needs the Mac, so it's left for the Mac session, with Nevada's OK.
