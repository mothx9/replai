# Changelog

Externally meaningful changes are recorded here. This is not a release ledger
or an implementation diary; [project status](ROADMAP.md) owns current limits and
next work. All entries below are unreleased and carry no compatibility promise.

## Unreleased

- Validated multiline input now handles Tab indentation in continuation-line
  whitespace prefixes, inserting spaces to the next four-cell stop atomically.
  Completion after text and menu navigation retain priority in their contexts.
  Direct submission, the minimal blocking tier and C ABI 1 are unchanged.

- Added native revision-bound editor style spans and non-canonical hints. Hosts
  supply one bounded AnalysisPresentation; stale results have no display effects.
  Completion/validation retain their separate insertion/submission contracts.
  Plain hints remain explicitly marked; C ABI 1 has no new entry points.

- Added opt-in native revision-bound submission validation with host-owned
  Complete/Incomplete/Invalid decisions, bounded safe diagnostics and logical
  multiline navigation. `Event::SubmissionRequested` extends the Rust enum;
  exhaustive event matches need an additional arm. Default direct submission,
  the minimal blocking entry and C ABI 1 retain their prior behavior. No parser,
  validator executor, general highlighting or validation C entry is added.

- Added native blocking reads and external readiness/deadline/resize driving over
  the retained Interaction engine. Existing session polling and C ABI 1 remain
  available. New terminal fact/policy admission separates optional styling/paste
  from required editing features. Added `Error::CapabilityMismatch`; exhaustive
  native Rust error matches need an additional arm. No driven C entry is added.

- Added safe structured Rust presentation: headings, facts, lists, responsive
  tables, literal blocks and severity notices share inline roles, plain rendering
  and coordinated draft-preserving output. Added composed prompts/continuations
  and explicit role styles while preserving simple prompts and C ABI 1.

- Made interaction state/events available independently of the Linux backend.
  The platform-neutral engine and semantic render pipeline retain the existing
  Linux Rust/C behavior; system acquisition outside Linux remains unimplemented.

- Introduced an independent Rust terminal interaction library with bounded
  Unicode/grapheme editing, draft-preserving history, host completion, multiline
  paste, typed outcomes, Linux terminal restoration and coordinated output.
- Added terminal-native prompt/style roles, continuation and multirow redraw,
  respecting color-disable rules and the terminal's default background.
- Replaced lifetime-bound `Terminal<'a>` with owned `Interaction` composition
  to support explicit close/reopen and movable owners. This breaks the initial
  experimental Rust composition API.
- Added pre-release C ABI 1 with explicit handle/FD/buffer ownership, staged
  static/shared artifacts and pkg-config integration through the public Rust API.
- Renamed the project, Rust package and native symbols/artifacts from REPLIA to
  REPLAI. Consumers of the previous spelling must update includes, symbols,
  package names and linkage together; no compatibility aliases are supplied.
