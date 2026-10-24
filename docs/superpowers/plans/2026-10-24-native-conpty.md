# Native image previews through Windows RMUX

Goal: display Yazi's full-resolution Sixel images inside RMUX while preserving
foreground process tracking, session lifecycle and transactional installation.

Use Microsoft's pinned Microsoft.Windows.Console.ConPTY 1.24.261001001 pair.
Archive SHA-256: 4d6aaddc1d2385c9f5897df28f33879f699f8f2783315d5204cf3d8c3616ac5f.
Load only an absolute sibling conpty.dll, with its paired OpenConsole.exe;
retain the DLL for every HPCON operation. Keep system API fallback when absent.

1. Reproduce direct system-ConPTY SIXEL loss. Verify the bundled runtime passes
   the same producer test before changing packaging or terminal detection.
   Files: rmux-pty Windows runtime.rs/pty.rs/mod.rs, Cargo.toml;
   tests/windows_attach_exit.rs. Run the named bundled-ConPTY regression.
2. Verify real RMUX attach delivery and Yazi selection. Advertise Sixel only
   when both the Windows Terminal outer and bundled runtime are available.
   Retain the existing allow-passthrough gate. Test missing runtime and generic
   Windows terminals as well as successful direct/attached delivery.
3. Package the signed runtime pair beside each executable, include Microsoft's
   license notice, checksum every staged file, and install/rollback companions
   with the existing file transaction. Prepare test runtimes in fork CI.
   Run package, installer-lock/rollback, process-title and capability regressions.
4. After review and green fork CI, publish a personal release and update
   chezmoi's immutable release/component hashes. Remove the temporary pane
   marker suppression only with the native-capable version available.

No live user sessions may be killed. A 0.9 daemon cannot use a 0.10 client;
installation must wait for the user to close those sessions.

Validation: direct and attached Sixel regression tests passed with staged native
runtime; six client-capability and six fork-workflow tests passed. A local
Yazi 26.5.6 probe on brand-logo-design.png emitted raster q"1;1;1200;675 through
RMUX attach after clearing the temporary config override. Debug archive binary,
daemon, installer lock, rollback and concurrency checks passed. Existing live
0.9 sessions were not touched.
