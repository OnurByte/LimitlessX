# Review, TypeScript and repository delivery

User requests implementation through push and merge. Keep existing Rust domain/runtime/storage and popup-only architecture.

1. Review current source and independent reviewer findings. Verify identity fallback, automatic unique counts, metadata ownership, navigation and media.
2. Move handwritten browser adapters to strict TypeScript. Build them to packaged JS through pinned tooling; keep Rust responsible for business decisions. Typecheck in xtask and CI. Generated wasm-bindgen glue remains JS.
3. Inspect native X media behavior and provide a reliable packaged player using public media URLs, with accessible controls and no X session credentials or remote executable player. Preserve playback on incremental feed updates. Test controls and media failure.
4. Align popup with design-reference.md (monochrome, ordinary typography, direct copy, functional hierarchy). Keep product mark and By Onur signature.
5. Rewrite README with logo, exact motto, honest browser badges, installation/build instructions, privacy/source limits and open-source/MIT emphasis. No fake store badges or installation counts.
6. Run native tests, strict TS, native/WASM clippy, packages, Chromium and Firefox browser checks, actual Helium inspection. Fix material review findings and document evidence.
7. Commit, push branch, open/attach PR, inspect CI, resolve failures and merge when green. Verify merged remote revision and clean local state. Do not mark goal complete before all steps are evidenced.
