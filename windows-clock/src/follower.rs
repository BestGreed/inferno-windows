//! In-process follower shared by the diagnostic CLI and the Windows audio backend.
use crate::{
    local::LocalClock,
    ptp::{self, Identity, Message, Pairer},
    servo::{Config, Servo, Snapshot},
};
use std::{
    error::Error,
    net::{Ipv4Addr, SocketAddrV4},
};
pub fn follow(
    args: &[String],
    multicast: bool,
    stop: &std::sync::atomic::AtomicBool,
    mut publish: impl FnMut(Option<crate::servo::Mapping>),
) -> Result<(), Box<dyn Error>> {
    if args.len() != 4 {
        return Err(
            "usage: --follow-unicast INTERFACE_IPV4 MASTER_IPV4 LOCAL_MAC_12HEX SECONDS".into(),
        );
    }
    let interface: Ipv4Addr = args[0].parse()?;
    let master: Ipv4Addr = args[1].parse()?;
    let mac = &args[2];
    if mac.len() != 12 || !mac.is_ascii() {
        return Err("MAC must be 12 hex ASCII characters".into());
    }
    let mut uuid = [0u8; 6];
    for i in 0..6 {
        uuid[i] = u8::from_str_radix(&mac[i * 2..i * 2 + 2], 16)?;
    }
    let seconds: u64 = args[3].parse()?;
    if seconds > 120 {
        return Err("duration must be 1..120 seconds".into());
    }
    let local = Identity { uuid, port: 1 };
    let clock = LocalClock::new()?;
    // Exclusive binds: never enable address reuse alongside DVS on these ports.
    let event = crate::net::bind_exclusive_v4(319)
        .map_err(|e| format!("PTP event bind failed (DVS may own port): {e}"))?;
    let general = crate::net::bind_exclusive_v4(320)?;
    // Bind TX to the selected local address, independently of the wildcard
    // multicast receive sockets. Dante peers also use nonstandard source ports.
    let transmit = std::net::UdpSocket::bind((interface, 0))?;
    crate::net::multicast_interface(&transmit, interface)?;
    println!("transmit_endpoint={}", transmit.local_addr()?);
    for socket in [&event, &general] {
        socket.join_multicast_v4(&Ipv4Addr::new(224, 0, 1, 129), &interface)?;
        socket.set_nonblocking(true)?;
    }
    let mut pairer = Pairer::new(local);
    let mut parent_packet: Option<[u8; 124]> = None;
    let mut servo = Servo::new(Config::default());
    let mut snapshot = Snapshot::default();
    let end = clock.now_ns() + seconds as i64 * 1_000_000_000;
    let mut counts = [0u64; 5];
    let mut buffer = [0u8; 2048];
    let mut last_report = 0;
    println!("experimental_follower=true multicast_delay={multicast} master={master} no_master_election=true software_timestamps=true");
    while (seconds == 0 || clock.now_ns() < end) && !stop.load(std::sync::atomic::Ordering::Relaxed)
    {
        for (is_event, socket) in [(true, &event), (false, &general)] {
            // Bound work per iteration so malformed traffic cannot starve expiry/exit.
            for _ in 0..64 {
                let (n, source) = match socket.recv_from(&mut buffer) {
                    Ok(v) => v,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(e) => return Err(e.into()),
                };
                let received = clock.now_ns();
                // Dante hardware may source PTP from 1031/1032. Validate the
                // destination socket, pinned peer IP and PTP port identity;
                // the peer's UDP source port is not required to be 319/320.
                if source.ip() != std::net::IpAddr::V4(master) {
                    continue;
                }
                counts[0] += 1;
                match ptp::decode(&buffer[..n], &ptp::DEFAULT_SUBDOMAIN) {
                    Ok(Message::Sync { source, sequence }) if is_event => {
                        if pairer.sync(source, sequence, received).is_err() {
                            counts[4] += 1;
                        } else {
                            parent_packet = Some(buffer[..124].try_into().unwrap());
                        }
                    }
                    Ok(Message::FollowUp {
                        source,
                        associated_sequence,
                        origin_ns,
                    }) if !is_event => {
                        let t3 = clock.now_ns();
                        if let Ok(seq) =
                            pairer.follow_up(source, associated_sequence, origin_ns, t3)
                        {
                            let request = ptp::delay_request_with_parent(
                                local,
                                seq,
                                &ptp::DEFAULT_SUBDOMAIN,
                                parent_packet.as_ref().ok_or("missing parent dataset")?,
                                pairer
                                    .request_origin_estimate()
                                    .ok_or("missing request timestamp")?,
                            )?;
                            if counts[1] == 0 {
                                println!(
                                    "request_hex={}",
                                    request
                                        .iter()
                                        .map(|v| format!("{v:02x}"))
                                        .collect::<String>()
                                );
                            }
                            if transmit
                                .send_to(
                                    &request,
                                    SocketAddrV4::new(
                                        if multicast {
                                            Ipv4Addr::new(224, 0, 1, 129)
                                        } else {
                                            master
                                        },
                                        319,
                                    ),
                                )
                                .is_err()
                            {
                                pairer.send_failed();
                                counts[4] += 1;
                            } else {
                                counts[1] += 1;
                            }
                        }
                    }
                    Ok(Message::DelayResponse {
                        source,
                        requester,
                        request_sequence,
                        receipt_ns,
                    }) => {
                        if requester == local {
                            println!("own_delay_response sequence={request_sequence} socket_event={is_event}");
                        }
                        match pairer.response(
                            source,
                            requester,
                            request_sequence,
                            receipt_ns,
                            received,
                        ) {
                            Ok(exchange) => match servo.observe(exchange, received) {
                                Ok(_) => {
                                    counts[2] += 1;
                                    snapshot.publish(servo.mapping(received));
                                }
                                Err(reason) => {
                                    counts[3] += 1;
                                    snapshot.publish(servo.mapping(received));
                                    println!("observation_rejected={reason}");
                                }
                            },
                            Err(_) => counts[4] += 1,
                        }
                    }
                    Err(_) => counts[4] += 1,
                    _ => {}
                }
            }
        }
        let now = clock.now_ns();
        publish(snapshot.load(now));
        if now - last_report >= 1_000_000_000 {
            println!("packets={} requests={} observations={} rejected={} unmatched={} locked={} rate_ppb={} residual_ns={:.1}",counts[0],counts[1],counts[2],counts[3],counts[4],snapshot.load(now).is_some(),servo.status.rate_ppb,servo.status.residual_ns);
            last_report = now;
        }
        crate::net::wait_readable(&event, &general)?;
    }
    publish(None);
    if !stop.load(std::sync::atomic::Ordering::Relaxed) && snapshot.load(clock.now_ns()).is_none() {
        return Err("no currently valid clock lock; do not use for TX".into());
    }
    Ok(())
}
