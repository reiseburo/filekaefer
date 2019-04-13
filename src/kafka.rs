///
/// The Kafka module provides the simple client shims for rdkafka to be used alongside the
/// StatefulFile struct
///

use std::io::Result;
use futures::Future;

use rdkafka::config::ClientConfig;
use rdkafka::message::{BorrowedMessage, OwnedHeaders};
use rdkafka::producer::{FutureProducer, FutureRecord, DeliveryResult};
use rdkafka::util::get_rdkafka_version;

pub struct Kafka {
    producer: FutureProducer,
}

impl Kafka {
    pub fn new(brokers: &str) -> Self {
        Kafka {
            producer: ClientConfig::new()
                .set("bootstrap.servers", brokers)
                .set("produce.offset.report", "true")
                .set("message.timeout.ms", "5000")
                .create()
                .expect("Producer creation error"),
        }
    }

    pub fn write(&self, topic: &str, message: &str) {
        /// NOTE: Key should be an optional parameter perhaps
        let record = FutureRecord::to(topic)
            .payload(message)
            .key(message);

        self.producer.send(record, 0).wait();
    }
}
