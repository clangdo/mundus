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
use std::iter::FromIterator;

const PREFIX: char = '-';

const HELP: &str = "```
Commands:
-help|-?             Print this screen
-info|-about         Print the about message
-ping|-pong          Health check, will respond with inverse
-roll expression     Roll some dice and put the results together in
                     a mathematical expression

                     Dice are specified in a \"2d10\" format, where the
                     first number is the quantity, and the second is 
                     the number of sides. Constants are also allowed in
                     expressions. The supported operators are currently
                     + (add), - (subtract), * (multiply), and / (divide)
```
";

fn roll(args: &mut str::SplitWhitespace<'_>) -> String {
    let mut rng = thread_rng();
    let expr_str = String::from_iter(args);
    match Expression::from_str(expr_str.as_str()) {
        Ok(mut expr) => {
            expr.determine(&mut rng);
            if let Ok(result) = expr.resolve(&mut rng) {
                String::from(format!("`{} = {}`", expr, result))
            } else {
                String::from(format!("`{} = unknown`", expr))
            }
        },
        Err(why) => String::from(format!("`{}`", why.to_string())),
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
                "ping" => Some(String::from("`pong`")),
                "pong" => Some(String::from("`ping`")),
                "help" | "?" => Some(String::from(HELP)),
                "about" | "info" => Some(String::from("`Hi. I'm Mundus, the world-bot.`")),
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
