# Native Windows Dante loop test

This experimental port uses an in-process QPC/PTPv1 follower and the existing
Inferno discovery, control, TX and RX implementation. No DVS or ASIO API is used.
It does not elect a master or change the system clock.

Start from a recursive clone, using a standard Windows Rust toolchain (MSVC
or MinGW) and its linker. The three submodules are needed for workspace
dependency resolution, even though the Windows clock does not use usrvclock.
Build only the Windows host (not the ALSA workspace package):

```
cargo build --locked --release -p inferno_aoip --example windows_loopback
```

Run with an explicitly selected interface, master and interface MAC:

```
windows_loopback.exe INTERFACE_IPV4 MASTER_IPV4 LOCAL_MAC_12HEX SECONDS
```

Duration is 1..1800 seconds after audio startup. Clock acquisition has a separate
30-second limit. Ctrl+C stops the audio test. The PTP worker releases its sockets
when the host shuts down. DVS must not own PTP ports 319/320 for this test.

The device is **Inferno Windows Loop Test**, 48 kHz, 2 TX / 2 RX. TX channels
contain periodic 375 Hz and 562.5 Hz sine waves at -30 dBFS. The operator routes
TX 1/2 through another computer and returns those channels to RX 1/2 in Dante
Controller. No automatic subscriptions are created by the test entry point.
Inferno's existing saved-subscription behavior still applies on later runs.

Allow this executable's LAN UDP traffic on the test adapter: PTP 319/320,
mDNS 5353, test control ports **18000..18003**, and negotiated media ports.
Alternate control ports use Inferno's existing ALT_PORT mechanism and avoid
Dante conmon's wildcard listeners at 8700/8800; PROCESS_ID=1 separates the test
device identity. Existing services need not be stopped.

The terminal reports clock validity, RX callback frames and nonzero samples.
Zero received frames means no return audio has been observed. Nonzero samples
alone do not prove correct channel mapping, frequencies, continuity or latency.
A steady periodic tone cannot uniquely determine end-to-end latency.

The Windows backend revokes expired mappings and forwards clock loss to TX/RX.
TX resumes using a fresh flow timestamp after reacquisition. Software timestamp
accuracy, scheduling jitter, channel integrity and sustained real audio delivery
still require hardware-loop testing. Do not interpret servo residual as absolute
synchronization accuracy. The Linux usrvclock integration test remains Unix-only.

Windows dependency fix: vendored netdev 0.32.0 retains its MIT license, with a
single SendARP output-length guard documented in vendor/netdev/PATCH.md.

## Automated checks without audio hardware

```
cargo test --locked -p inferno-windows-clock
cargo test --locked -p inferno_aoip --lib
cargo run --locked --release -p inferno-windows-clock -- --simulate
cargo run --locked --release -p inferno-windows-clock -- --probe
```

The mock-master test binds ephemeral loopback ports, not 319/320. The probe
only reads QPC. Neither starts audio nor adjusts the system clock or scheduler.
On Linux, the clock crate uses an Instant-based diagnostic fallback; the audio
library continues to use usrvclock. Linux diagnostic tests do not exercise the
Windows socket implementation. The workflow checks both platforms; it never
runs the physical-network follower or audio example.

Physical-peer observations made during earlier development are separate from
these checks. This extracted contribution still needs sustained TX/RX hardware
validation. It makes no absolute synchronization-accuracy, complete Controller
telemetry or stable 250 us TX claim. Independent source-clock drift compensation
is not implemented.
