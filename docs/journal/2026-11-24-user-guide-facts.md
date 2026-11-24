# 2026-11-24: The user guide's fact sheet (M7 slice 12, #281)

- **Nevada asked for the fact sheet** while away at lunch, after #255. The design session will write the user guide, a page on the website, from it (M7 plan, question 8).
- **How it was made:** Claude Code had a helper agent draft it from the code, the journal, the functional spec, the user flows, the decision records and the open issues, with a source for every fact. Claude Code then checked it. `docs/user-guide-facts.md` describes `main` on 2026-11-24, which is 0.5 plus #217's, #264's and #276's fixes, and it says where 0.5 differs. It covers everything the milestone plan lists for the guide, plus the basics, known issues, what isn't built yet, and a table of which cloud services were tested where.
- **Twelve open questions for the design session,** where sources disagree or the code differs from the spec. Some examples:
  - 0.5 ships no Intel Mac build, though the technical design says Intel is supported.
  - The Editor doesn't take the rating and tag keys the user flows say work in every view.
  - The Viewer's key hint leaves out keys that work there.
  - Google Drive on Windows is described as handled in two documents, but it isn't yet.
- **Settled while checking:** the Windows install folder is `%LOCALAPPDATA%\Seura Photos`, as decision 0013 says and as Nevada's PC has it. Decision 0015's alternatives give an older path.
- **Filed from it: #289.** On the Mac, Seura assumes every drive has a Trash. On a network share, Delete would show a failure rather than the spec's explanation, and clearing a last rating there would fail. Nothing is lost either way, and it hasn't been measured.
- **Nevada decided #291 won't be fixed** (a folder saved through a link and missing at launch misses live changes once it comes back, until a relaunch; found by the Mac session checking #288). It's rare enough to document instead, so it's in the fact sheet's known issues and the issue is closed as not planned.
