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

mod tail;
mod kafka;
use kafka::Kafka;

fn produce(brokers: &str, topic_name: &str) {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("produce.offset.report", "true")
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    // This loop is non blocking: all messages will be sent one after the other, without waiting
    // for the results.
    let futures = (0..5)
        .map(|i| {
            // The send operation on the topic returns a future, that will be completed once the
            // result or failure from Kafka will be received.
            producer.send(
                FutureRecord::to(topic_name)
                    .payload(&format!("Message {}", i))
                    .key(&format!("Key {}", i)),
                   // .headers(OwnedHeaders::new()
                   //     .add("header_key", "header_value")),
                0
            )
                .map(move |delivery_status| {   // This will be executed onw the result is received
                    println!("Delivery status for message {} received", i);
                    delivery_status
                })
        })
        .collect::<Vec<_>>();

    // This loop will wait until all delivery statuses have been received received.
    for future in futures {
        println!("Future completed. Result: {:?}", future.wait());
    }
}

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
   //     .arg(Arg::with_name("log-conf")
   //          .long("log-conf")
   //          .help("Configure the logging format (example: 'rdkafka=trace')")
   //          .takes_value(true))
   //     .arg(Arg::with_name("topic")
   //          .short("t")
   //          .long("topic")
   //          .help("Destination topic")
   //          .takes_value(true)
   //          .required(true))
   //     .get_matches();

    //setup_logger(true, matches.value_of("log-conf"));

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
    //produce(brokers, topic);
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
