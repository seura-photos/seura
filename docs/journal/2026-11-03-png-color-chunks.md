# A PNG's gamma or chromaticities on their own (#188, #219)

- **The question.** #188 named PNG's `gAMA` and `cHRM` together, and Claude Code also made either one count alone, with sRGB's for the missing part. It asked Nevada, since this changes how some PNGs look.
- **The tradeoff.** A gamma of 2.2 alone darkens deep shadows slightly compared with sRGB: a stored 30 shows as about 24, and 128 as about 129, worked out by hand. A file mislabeled that way would look a little too dark, and Seura can't tell it from a true one. Files with a gamma of 1.0 or 1.8 were clearly too dark as sRGB, and now look right: a stored 128 shows as about 188 and 146.
- **Nevada decided to follow the file,** so #219 stays as it is.
- **The user guide will explain this (Nevada).** It's added to M7's user guide in the milestone plan.
- **moxcms's wrong extended-range values won't be reported to the project for now (Nevada).** Seura doesn't use that feature.
