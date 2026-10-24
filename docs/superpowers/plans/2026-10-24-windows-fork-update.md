# Update the Windows fork to official RMUX 0.10.0

Goal: retain Windows foreground process tracking on the official v0.10.0 base,
keep the existing personal release path, and verify Yazi's Chafa fallback.
Execute inline in the existing isolated worktree; preserve the old fork branch.

## Constraints

- Base: dfd68c774ca0f4212139a21d37d09c90f75f8bd7 (official v0.10.0).
- Carry 43d90977 (foreground process tracking), d5c55685 and 046e9d69 (release CI).
- Do not carry 6f8639e9: the status mouse-target fix is already upstream.
- Preserve the current installation and running user sessions during validation.
- Use pinned archives and component hashes for any subsequent chezmoi update.

## Tasks

1. Port foreground tracking and its existing tests. First reproduce the missing
   tracking against the official binary; then migrate the patch, resolve API
   changes, run its Windows integration tests and relevant crate tests.
   Files: crates/rmux-os/src/process*.rs, crates/rmux-pty/src/{child.rs,backend/windows/spawn.rs},
   crates/rmux-server/src/{format_runtime/process.rs,pane_terminal*.rs,pane_terminals/pane_access.rs},
   tests/windows_attach_exit.rs and associated existing tests.
2. Carry the personal release workflow and its contract test. Match current
   upstream toolchain/package commands. Run the workflow specification test,
   formatting and Windows package build checks. Commit the resulting update.
3. Verify the resulting fork with the managed RMUX configuration and a fresh
   Yazi process with YAZI_FILE_ONE set. Review the complete patch before publishing
   or installing. Update chezmoi's pin only after a verified release exists.
4. User diagnostics confirm Yazi selects Sixel inside RMUX, while RMUX's client
   feature list omits it. Advertise Sixel for Windows Terminal in the shared CLI
   capability detector and its server fallback. Extend their existing tests;
   leave non-Windows-Terminal capability lists unchanged. Document the supported
   Windows Terminal minimum (1.22.10352.0). Verify protocol bytes across a Windows
   ConPTY attach before claiming preview repair.

## Review focus

- Foreground tracking must return to Nu/cmd after the child exits.
- Processes belonging to another pane must not influence its title.
- Upstream mouse-target behavior must remain intact.
- Release workflows must remain restricted to personal tags and the fork.
- 0.10.0 uses wire version 8: never replace/kill the user's running 0.9 daemon.

## Evidence already collected

Official archive SHA-256: e315e2d51d927ba9621732812c0f932c862d05f4b677dbf3cab76f0d27372a70.
Official 0.10.0 passes chezmoi's rmux-config tests. Both it and the installed
0.9.0 fork draw the supplied PNG through Chafa when YAZI_FILE_ONE is present.
The official Windows Terminal capability detection still omits Sixel.

Update: the user's actual pane has YAZI_FILE_ONE and working MIME detection.
Its debug report selects Microsoft/Sixel, confirming the capability mismatch.
The missing environment variable in the isolated test was a separate condition.
