extern crate clap;
extern crate futures;
extern crate rdkafka;

use clap::{App, Arg};
use futures::*;

use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;

use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::get_rdkafka_version;

use rdkafka::message::OwnedHeaders;

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
             .help("Watch the given file")
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

    let mut f = File::open(watches)?;
    let reader = BufReader::new(f);

    for line in reader.lines() {
        println!("{}", line.unwrap());
    }
    Ok(())

    //produce(brokers, topic);
}
