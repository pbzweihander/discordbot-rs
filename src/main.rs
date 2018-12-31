#[cfg(feature = "use-dotenv")]
extern crate dotenv;
extern crate serenity;

#[cfg(feature = "use-dotenv")]
use dotenv::dotenv;

use std::env;

use serenity::{
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

struct Handler;

impl EventHandler for Handler {
    fn message(&self, _: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say("Pong!") {
                eprintln!("Error sending message: {:?}", why);
            }
        }
    }
}

fn main() {
    #[cfg(feature = "use-dotenv")]
    dotenv().expect("Err initializing dotenv");

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let mut client = Client::new(&token, Handler).expect("Err creating client");

    if let Err(why) = client.start() {
        eprintln!("Client error: {:?}", why);
    }
}
