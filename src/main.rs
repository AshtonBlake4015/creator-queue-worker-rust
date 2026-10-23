mod queue_worker;

use queue_worker::{decide, CreatorJob, InfraiQueue};

fn main() {
    let queue = InfraiQueue::from_env().expect("set INFRAI_API_KEY");
    let request = queue.consume_request("creator-jobs", 4);
    println!("{} {}", request.method, request.path);

    let job = CreatorJob {
        kind: "digital_asset".into(),
        asset_id: "asset-204".into(),
        subscriber_id: "subscriber-18".into(),
    };
    match decide(&job) {
        Ok(action) => {
            println!("action: {:?}", action);
            let ack = queue.ack_request("creator-jobs", "message-204");
            println!("{} {}", ack.method, ack.path);
        }
        Err(error) => {
            let nack = queue.nack_request("creator-jobs", "message-204");
            println!("job rejected: {:?}; {} {}", error, nack.method, nack.path);
        }
    }
}
