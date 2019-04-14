///
/// The configuration module is responsible for parsing both the Kinesis Agent compatible
/// configuration and other formats as well
///

use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub struct KinesisConfig {
    #[serde(rename = "checkpointFile")]
    checkpoint: String,
    #[serde(rename = "kinesis.endpoint")]
    endpoint: String,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_deserializes_no_flows() {
        let buf = r#"
{
    "checkpointFile" : "/tmp/foo.log",
    "kinesis.endpoint" : "someendpoint:9092"
}"#;
        let des: KinesisConfig = serde_json::from_str(&buf).unwrap();
        assert_eq!(des.checkpoint, "/tmp/foo.log");
        assert_eq!(des.endpoint, "someendpoint:9092");
    }
}
