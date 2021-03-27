mod responding;

use std::env;

use serenity::prelude::*;

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN")
        .expect("Mundus requires a token!");

    let mut client = Client::builder(&token)
        .event_handler(responding::Responder)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Error starting client: {:?}", why);
    }
}
