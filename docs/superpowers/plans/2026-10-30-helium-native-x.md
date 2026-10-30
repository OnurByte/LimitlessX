# Helium native X fidelity and installation

Goal: Match the user's actual Helium X profile, detailed post and reply layouts; install and verify LimitlessX in that same browser.
Authority: Latest user request supersedes estimated styling. Current direction still requires automatic X DOM injection, popup only, supplied SVG, and no use of X session credentials by the extension.

- [x] Connect Chrome DevTools MCP to the existing Helium profile after browser remote-debugging permission.
- [x] Install current unpacked extension in Helium and verify enabled ID.
- [x] Capture public profile, post and reply computed styles, native icon paths and geometry.
- [x] Reproduce and fix actual Account Withheld DOM heading detection.
- [ ] Add typed optional profile fields, metrics, quoted posts and conversation replies/pagination; preserve tombstone/error distinctions.
- [ ] Match native layout: 600px column, 16px gutters, 40px avatars, 8px gap, 15/20px replies, full-width 17/24px detailed text, native header/banner/counts and native icon metrics.
- [ ] Add regression tests for real notice, conversation identity/pagination, metadata nullability, DOM text safety and native computed geometry.
- [ ] Rebuild, reload extension through browser MCP and verify automatic display on the actual withheld account; verify profile, post and replies in Helium.
- [ ] Capture evidence and compatibility limits; preserve user's unrelated browser tabs and settings.

Current extension ID: lbeeopacdoaccgmepkfhilamcdnhibii.
Public reference pages: https://x.com/Osimhen_31 and https://x.com/Burhimum/status/2115761952937128381.
Live restriction reference: https://x.com/ekrem_imamoglu, DOM text begins Account Withheld@ekrem_imamoglu has been withheld in TR.
