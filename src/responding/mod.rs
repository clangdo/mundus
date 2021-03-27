mod rolling;

use rand::prelude::*;
use rolling::Expression;
use serenity::{
    async_trait,
    model::{id::ChannelId, channel::Message, event::ResumedEvent, gateway::Ready},
    prelude::*,
};
use std::fmt;
use std::str;

const PREFIX: char = '-';

fn roll(args: &mut str::SplitWhitespace<'_>) -> String {
    let mut rng = thread_rng();
    if let Some(expr_str) = args.next() {
        match Expression::from_str(expr_str) {
            Ok(expr) => {String::from(format!("{}", expr.resolve(&mut rng).unwrap_or(0)))},
            Err(why) => String::from(why.to_string()),
        }
    } else {
        String::from("No expression specified for the roll, you should give it in the form '1d6 + 4', '2d20 * 1d4', etc.")
    }
}

pub struct Responder;

impl Responder {
    async fn try_send(on: ChannelId, what: impl fmt::Display, ctx: Context) {
        if let Err(why) = on.say(&ctx.http, what).await {
            println!("Error sending message: {:?}", why);
        }
    }

    async fn unwrap_command(message: &str) -> Option<str::SplitWhitespace<'_>> {
        if let Some(command) = message.strip_prefix(PREFIX) {
            Some(command.split_whitespace())
        } else {
            None
        }
    }
}

#[async_trait]
impl EventHandler for Responder {
    async fn message(&self, ctx: Context, msg: Message) {
        if let Some(mut command) = Self::unwrap_command(&msg.content).await {
            if let Some(response) = match command.next().unwrap_or("help") {
                "ping" => Some(String::from("pong")),
                "about" => Some(String::from("Hi. I'm Mundus, the world-bot")),
                "roll" => Some(roll(&mut command)),
                _ => None,
            } {
                Self::try_send(msg.channel_id, response, ctx).await;
            }
        }
    }

    async fn resume(&self, _ctx: Context, _resume: ResumedEvent) {
        println!("Resumed");
    }
    
    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected.", ready.user.name);
    }
}
