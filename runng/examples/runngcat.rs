//! Rust nngcat.  See [nngcat](https://nanomsg.github.io/nng/man/v1.1.0/nngcat.1).
//! `cargo run --example runngcat`

use clap::{App, Arg, ArgGroup, ArgMatches};
use env_logger::{Builder, Env};
use log::info;
use runng::{protocol::Subscribe, *};
use std::{thread, time::Duration};

fn connect<T>(socket: T, url: &str, is_dial: bool) -> NngResult<T>
where
    T: Dial + Listen,
{
    if is_dial {
        socket.dial(url)
    } else {
        socket.listen(url)
    }
}

const PROTOCOLS: &[&str] = &["req0", "rep0", "pub0", "sub0", "push0", "pull0"];

fn main() -> NngReturn {
    Builder::from_env(Env::default().default_filter_or("debug"))
        .try_init()
        .unwrap_or_else(|err| println!("env_logger::init() failed: {}", err));

    let matches = get_matches();
    let is_dial = matches.is_present("dial");
    let url = matches
        .value_of("dial")
        .or_else(|| matches.value_of("listen"))
        .unwrap()
        .to_owned();

    let mut threads = Vec::new();
    if matches.is_present("req0") {
        let matches = matches.clone();
        let url = url.clone();
        let sock = protocol::Req0::open()?;
        let thread = thread::spawn(move || -> NngReturn {
            let sock = connect(sock, &url, is_dial)?;
            let reply = handle_data(&matches)?;
            handle_delay(&matches);
            if let Some(interval) = matches.value_of("interval") {
                let interval = interval.parse::<u64>().unwrap();
                loop {
                    sock.send(reply.dup()?)?;
                    let msg = sock.recv()?;
                    handle_received_msg(&matches, msg);
                    thread::sleep(Duration::from_secs(interval));
                }
            } else {
                sock.send(reply.dup()?)?;
                let msg = sock.recv()?;
                handle_received_msg(&matches, msg);
            }

            Ok(())
        });
        threads.push(thread);
    }
    if matches.is_present("rep0") {
        let matches = matches.clone();
        let url = url.clone();
        let sock = protocol::Rep0::open()?;
        let thread = thread::spawn(move || -> NngReturn {
            let sock = connect(sock, &url, is_dial)?;
            let reply = handle_data(&matches)?;
            //handle_delay(&matches);
            loop {
                let msg = sock.recv()?;
                handle_received_msg(&matches, msg);
                sock.send(reply.dup()?)?;
            }
        });
        threads.push(thread);
    }
    if matches.is_present("pub0") {
        let matches = matches.clone();
        let url = url.clone();
        let sock = protocol::Pub0::open()?;
        let thread = thread::spawn(move || -> NngReturn {
            let sock = connect(sock, &url, is_dial)?;
            let msg = handle_data(&matches)?;
            handle_delay(&matches);
            if let Some(interval) = matches.value_of("interval") {
                let interval = interval.parse::<u64>().unwrap();
                loop {
                    sock.send(msg.dup()?)?;
                    thread::sleep(Duration::from_secs(interval));
                }
            } else {
                sock.send(msg.dup()?)?;
            }

            Ok(())
        });
        threads.push(thread);
    }
    if matches.is_present("sub0") {
        let matches = matches.clone();
        let url = url.clone();
        let sock = protocol::Sub0::open()?;
        let topic = matches
            .value_of("subscribe")
            .or(Some(""))
            .unwrap()
            .to_owned();
        let thread = thread::spawn(move || -> NngReturn {
            let sock = connect(sock, &url, is_dial)?;
            sock.subscribe(topic.as_bytes())?;
            let msg = sock.recv()?;
            handle_received_msg(&matches, msg);
            Ok(())
        });
        threads.push(thread);
    }
    if matches.is_present("push0") {
        let matches = matches.clone();
        let url = url.clone();
        let sock = protocol::Push0::open()?;
        let thread = thread::spawn(move || -> NngReturn {
            let sock = connect(sock, &url, is_dial)?;
            let msg = handle_data(&matches)?;
            handle_delay(&matches);
            if let Some(interval) = matches.value_of("interval") {
                let interval = interval.parse::<u64>().unwrap();
                loop {
                    sock.send(msg.dup()?)?;
                    thread::sleep(Duration::from_secs(interval));
                }
            } else {
                sock.send(msg.dup()?)?;
            }

            Ok(())
        });
        threads.push(thread);
    }
    if matches.is_present("pull0") {
        let matches = matches.clone();
        let url = url.clone();
        let sock = protocol::Pull0::open()?;
        let thread = thread::spawn(move || -> NngReturn {
            let sock = connect(sock, &url, is_dial)?;
            loop {
                let msg = sock.recv()?;
                handle_received_msg(&matches, msg);
            }
        });
        threads.push(thread);
    }

    threads
        .into_iter()
        .for_each(|thread| thread.join().unwrap().unwrap());

    Ok(())
}

fn handle_data<'a>(matches: &ArgMatches<'a>) -> NngResult<msg::NngMsg> {
    let mut msg = msg::NngMsg::create()?;
    if let Some(data) = matches.value_of("data") {
        msg.append_slice(data.as_bytes());
    }
    Ok(msg)
}

fn handle_delay<'a>(matches: &ArgMatches<'a>) {
    if let Some(delay) = matches.value_of("delay") {
        let delay = delay.parse::<u64>().unwrap();
        thread::sleep(Duration::from_secs(delay));
    }
}

fn handle_received_msg<'a>(_matches: &ArgMatches<'a>, msg: msg::NngMsg) {
    info!("Received {:?}", msg);
}

fn get_matches<'a>() -> ArgMatches<'a> {
    let mut args = App::new("runngcat")
        .arg(
            Arg::with_name("version")
                .short("V")
                .long("version")
                .help("Print the version and exit"),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Select verbose operation"),
        )
        .arg(
            Arg::with_name("silent")
                .short("q")
                .long("silent")
                .help("Select silent operation"),
        )
        .arg(
            Arg::with_name("subscribe")
                .long("subscribe")
                .takes_value(true)
                .value_name("TOPIC")
                .help("Subscribe to TOPIC"),
        )
        .arg(
            Arg::with_name("count")
                .long("count")
                .takes_value(true)
                .value_name("COUNT")
                .help("Limit the number of iterations when looping to COUNT iterations"),
        )
        // Peer selection options
        .arg(
            Arg::with_name("dial")
                .long("dial")
                .takes_value(true)
                .value_name("URL")
                .help("Connect to the peer at the address specified by URL"),
        )
        .arg(
            Arg::with_name("listen")
                .long("listen")
                .takes_value(true)
                .value_name("URL")
                .help(
                    "Bind to, and accept connections from peers, at the address specified by URL",
                ),
        )
        // .arg(
        //     Arg::with_name("connect-ipc")
        //     .short("x")
        //     .long("connect-ipc")
        //     .takes_value(true)
        //     .value_name("PATH")
        // )
        // .arg(
        //     Arg::with_name("bind-ipc")
        //     .short("X")
        //     .long("bind-ipc")
        //     .takes_value(true)
        //     .value_name("PATH")
        // )
        // .arg(
        //     Arg::with_name("connect-local")
        //     .short("l")
        //     .long("connect-local")
        //     .takes_value(true)
        //     .value_name("PORT")
        // )
        // .arg(
        //     Arg::with_name("bind-local")
        //     .short("L")
        //     .long("bind-local")
        //     .takes_value(true)
        //     .value_name("PORT")
        // )
        .group(
            ArgGroup::with_name("peer")
                .args(&["dial", "listen"])
                .multiple(false)
                .required(true),
        )
        // Receive options
        .arg(Arg::with_name("raw").long("raw"))
        .arg(
            Arg::with_name("receive-timeout")
                .long("receive-timeout")
                .takes_value(true)
                .value_name("SEC"),
        )
        // Transmit options
        .arg(
            Arg::with_name("data")
                .long("data")
                .short("D")
                .takes_value(true)
                .value_name("DATA")
                .help("Use DATA for the body of outgoing messages"),
        )
        .arg(
            Arg::with_name("interval")
                .long("interval")
                .short("i")
                .takes_value(true)
                .value_name("SEC"),
        )
        .arg(
            Arg::with_name("delay")
                .long("delay")
                .short("d")
                .takes_value(true)
                .value_name("SEC")
                .help("Wait SEC seconds before sending the first outgoing message"),
        );

    // Protocol selection options
    for protocol in PROTOCOLS {
        args = args.arg(Arg::with_name(protocol).long(protocol));
    }
    args.group(
        ArgGroup::with_name("protocols")
            .args(&PROTOCOLS)
            .multiple(true)
            .required(true),
    )
    .get_matches()
}
