extern crate clap;
extern crate futures;
extern crate inotify;

use clap::{App, Arg};
use futures::*;

use std::path::Path;
use std::fs::File;
use std::io::{Read, BufRead, Write, BufWriter};
use std::collections::HashMap;

use inotify::{Inotify, WatchMask, EventMask};

use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::get_rdkafka_version;

use rdkafka::message::OwnedHeaders;

mod config;
mod tail;
mod kafka;
use kafka::Kafka;

fn main() -> std::io::Result<()> {
    let matches = App::new("filekäfer")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or(""))
        .about("Little daemon for shipping logs to Kafka")
        .arg(Arg::with_name("brokers")
             .short("b")
             .long("brokers")
             .help("Broker list in kafka format")
             .takes_value(true)
             .default_value("localhost:9092"))
        .arg(Arg::with_name("watches")
             .short("w")
             .long("watch")
             .help("Watch the given files (space separated)")
             .takes_value(true)
             .required(true))
        .get_matches();

    let (version_n, version_s) = get_rdkafka_version();
    println!("rd_kafka_version: 0x{:08x}, {}", version_n, version_s);

    let topic = "test";
    let brokers = matches.value_of("brokers").unwrap();
    let watches = matches.value_of("watches").unwrap();

    println!("Should watch the file: {}", watches);

    let mut watcher = Inotify::init().expect("Inotify failed to initialize");
    let mut files = HashMap::new();

    for file_name in watches.split(" ") {
        let wd = watcher.add_watch(Path::new(&watches), WatchMask::MODIFY)
            .unwrap_or_else(|_| panic!("Failed to attach watcher to file: {}", &file_name));
        let fd = File::open(&file_name)
            .unwrap_or_else(|_| panic!("Failed to open file handle for: {}", &file_name));
        let mut sf = tail::StatefulFile::new(fd, file_name.to_string());
        sf.update_cursor();
        files.insert(wd, sf);
    }

    let k: Kafka = Kafka::new(brokers);

    let mut buffer = [0u8; 4096];
    loop {
        let events = watcher.read_events_blocking(&mut buffer)
            .expect("Failed to read inotify events");

        for event in events {
            if event.mask.contains(EventMask::MODIFY) {
                let sf = files.get_mut(&event.wd).unwrap();
                follow(sf, &k);
            }
        }
    }
    Ok(())
}

fn follow(sf: &mut tail::StatefulFile, k: &Kafka) {
    match sf.modification_type() {
        tail::ModificationType::Added => {}
        tail::ModificationType::Removed => {
            sf.reset_cursor();
        }
        tail::ModificationType::NoChange => {}
    }
    sf.update_metadata();
    sf.seek_to_cursor();

    for line in sf.fd.by_ref().lines() {
        k.write("test", &line.unwrap());
    }

    sf.update_cursor();
}

fn print_from_cursor(sf: &mut tail::StatefulFile) {
    let mut writer = BufWriter::new(std::io::stdout());
    for line in sf.fd.by_ref().lines().map(|l| l.unwrap()) {
        writer.write(line.as_bytes()).unwrap();
        writer.write(b"\n").unwrap();
    }
    writer.flush().unwrap();
}
