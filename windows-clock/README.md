# Minimal Windows clock prototype

This dependency-free crate now provides the in-process Windows clock backend
for `inferno_aoip`, alongside its diagnostic CLI. It belongs to the root workspace;
build it with `cargo build -p inferno-windows-clock`. Linux retains usrvclock.

## Implemented

- QPC/QPF local monotonic time, using integer nanosecond conversion.
- PTPv1 two-step Sync/Follow_Up/Delay_Resp decoding and Delay_Req encoding.
- Bounded pairing by master identity, requesting identity, sequence and age.
- A 32-observation affine time fit with delay rejection, residual/rate gates,
  minimum acquisition span, discontinuity reset and expiry.
- In-process atomic clock snapshots: one writer, bounded-retry readers; no
  allocation or mutex in a clock read. A per-consumer reader rejects backwards
  time. Exact affine inversion supports future TX deadline calculations.
- Opt-in IPv4 unicast-delay follower with an explicitly selected interface,
master IP and local MAC identity. It never sends Sync or joins master election.
- Windows SO_EXCLUSIVEADDRUSE before binding PTP ports. An existing DVS/PTP
  process is not stopped or displaced.

## Validation modes

From this folder, `cargo test --locked --offline` runs deterministic clock and
packet tests plus a UDP/QPC mock-master test on ephemeral loopback ports.

`cargo run --release -- --probe` checks local counter monotonicity and reports
read timings. This is not a network synchronization benchmark.

`cargo run --release -- --simulate` exercises synthetic offset and +80 ppm
frequency error. Synthetic error numbers do not indicate real PTP accuracy.

The UDP mock uses only ephemeral loopback ports. These tests do not require a physical PTP peer.

## Explicit experimental network mode

`inferno-windows-clock --follow-unicast INTERFACE_IPV4 MASTER_IPV4 LOCAL_MAC_12HEX SECONDS`

The duration must be 1..120 seconds. This binds UDP 319/320 exclusively, joins
224.0.1.129 on the chosen interface, accepts messages only from the specified
master IP (Sync on the event socket, Follow_Up on the general socket; Delay_Resp on either socket), pins the first Sync source identity, and sends
unicast Delay_Req to that master. Verify that the master supports this exchange.
The caller must ensure that the supplied MAC identifies the selected interface.
Port conflicts terminate the program; do not stop DVS just to run this command
without arranging the intended test state.

`--discover INTERFACE_IPV4 SECONDS` receives PTP packets without sending PTP
messages. `--follow-multicast` takes the same arguments as `--follow-unicast`
but sends Delay_Req to the default subdomain multicast group, TTL 1, on an
explicitly bound interface. The sender uses a separate ephemeral source port.
PTP source UDP ports need not be 319/320: actual hardware used 1031/1032.

Earlier development runs observed PTP exchanges and intermittent lock on a physical peer. Those observations are historical, not validation of this extracted contribution. The parent-dataset request encoder is experimental and its initial clock
quality fields are not calibrated measurements.

This mode uses user-space
software timestamps, Windows socket-readiness waiting, only `_DFLT` subdomain/zero epoch,
  two-step PTPv1 and unicast/multicast delay requests. It does not implement PTPv2, BMCA,
one-step Sync, redundant networks, path-asymmetry calibration, hardware
timestamping or automatic master replacement. Packet layout references were
checked against Wireshark's public `packet-ptp.c` field definitions; this crate
does not copy its implementation or DVS code.

The default acquisition gates (8 accepted samples spanning 2 seconds, 100 us
fit RMS, +/-500 ppm, 2-second expiry) are experimental settings, not promised
synchronization accuracy. Stable path asymmetry can bias an apparently clean
fit; passing gates does not establish absolute accuracy.

## Audio integration

The Windows `MediaClock` and TX/RX consumers now use this follower in process,
including expiry propagation and rate inversion. See ../WINDOWS-TESTING.md for
the two-channel synthetic source / return host. Independent source-rate
matching and sustained hardware audio validation are subsequent work.
