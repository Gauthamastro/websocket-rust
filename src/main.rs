
use std::env;
use std::time::SystemTime;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use websocket_lite::{Message, Opcode, Result};

async fn run() -> Result<()> {
    let url = env::args().nth(1).unwrap_or_else(|| "wss://stream.binance.com:9443/ws/btcusdt@trade".to_owned());
    let builder = websocket_lite::ClientBuilder::new(&url)?;
    let mut ws_stream = builder.async_connect().await?;
    let str : String = String::from(r#"{ "method": "SUBSCRIBE", "params": [ "btcusdt@trade" ], "id": 1 }"#);
     ws_stream.send(Message::text(str)).await;
    // ws_stream.send(Message::text(String::from("singh"))).await;

    let mut start_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_micros();
    let mut trade_counter: u32 = 0;
    let mut tps_sum:f64 = 0.0;
    loop {
        let msg: Option<Result<Message>> = ws_stream.next().await;

        let msg = if let Some(msg) = msg {
            msg
        } else {
            break;
        };

        let msg = if let Ok(msg) = msg {
            msg
        } else {
            //let _ = ws_stream.send(Message::close(None)).await;
            break;
        };

        match msg.opcode() {
            Opcode::Text => {
                // println!("{}", msg.as_text().unwrap());
                let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_micros();
                let instantaneous_tps = 1.0/(f64::from((current_time - start_time) as i32)/1000000.0);
                trade_counter = trade_counter +1;
                tps_sum = tps_sum + instantaneous_tps;
                println!("Instant Diff: {}, Average Diff: {}",instantaneous_tps, tps_sum/(trade_counter as f64));
                start_time = current_time;
            }
            Opcode::Binary =>  {},  // ws_stream.send(msg).await?,
            Opcode::Ping => ws_stream.send(Message::pong(msg.into_data())).await?,
            Opcode::Close => {
                let _ = ws_stream.send(Message::close(None)).await;
                break;
            }
            Opcode::Pong => {}
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    tokio::spawn(async {
        run().await.unwrap_or_else(|e| {
            eprintln!("{}", e);
        })
    })
        .await
        .unwrap();
}


