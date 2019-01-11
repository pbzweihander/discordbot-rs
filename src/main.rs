#[cfg(feature = "use-dotenv")]
use dotenv::dotenv;

use {
    backslash_z::*,
    futures::prelude::*,
    lazy_static::lazy_static,
    serenity::{builder::CreateMessage, model::channel::Message, prelude::*},
    std::{env, str::FromStr, thread},
    tokio,
};

lazy_static! {
    static ref APP_CONFIG: Config = Config {
        daummap_app_key: env::var("DAUMMAP_APP_KEY")
            .expect("Expected a daummap app key in the environment variables"),
    };
}

struct Handler;

fn format_resp(m: CreateMessage, resp: &Response) -> CreateMessage {
    match &resp {
        Response::Dictionary(ref search) => m.embed(|e| {
            e.description(if search.alternatives.is_empty() {
                String::new()
            } else {
                format!("Did you mean...\n{}", search.alternatives.join(", "))
            })
            .fields(search.words.iter().map(|w| {
                let mut body = String::new();
                if let Some(ref pronounce) = w.pronounce {
                    body.push_str(&pronounce);
                    body.push_str(" ");
                }
                body.push_str(&w.meaning.join(", "));
                (&w.word, body, false)
            }))
            .footer(|f| f.text("daumdic"))
        }),
        Response::AirPollution(ref status) => m.embed(|e| {
            e.description(format!("Station: {}", status.station_address))
                .fields(status.pollutants.iter().map(|p| {
                    (
                        &p.name,
                        format!(
                            "{}{} ({})",
                            p.level
                                .map(|f| f.to_string())
                                .unwrap_or_else(|| "--".to_string()),
                            p.unit,
                            p.grade,
                        ),
                        true,
                    )
                }))
        }),
        Response::HowTo(ref answer) => m.embed(|e| {
            e.title(&answer.link)
                .url(&answer.link)
                .description(format!(
                    "```\n{}\n```",
                    if answer.instruction.len() > 1000 {
                        &answer.instruction[0..1000]
                    } else {
                        &answer.instruction
                    }
                ))
                .footer(|f| f.text("howto"))
        }),
    }
}

impl EventHandler for Handler {
    fn message(&self, _: Context, msg: Message) {
        if let Ok(req) = Request::from_str(&msg.content) {
            let fut = req
                .request(&*APP_CONFIG)
                .map_err(|why| {
                    eprintln!("Error while requesting: {:?}", why);
                })
                .map(move |resp| {
                    if let Err(why) = msg.channel_id.send_message(|m| format_resp(m, &resp)) {
                        eprintln!("Error while sending message: {:?}", why);
                    }
                });
            thread::spawn(move || {
                tokio::run(fut);
            });
        }
    }
}

fn main() {
    #[cfg(feature = "use-dotenv")]
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment variables");

    let mut client = Client::new(&token, Handler).expect("Err creating client");

    if let Err(why) = client.start() {
        eprintln!("Client error: {:?}", why);
    }
}
