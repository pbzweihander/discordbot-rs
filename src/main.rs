#![recursion_limit = "128"]

#[cfg(feature = "use-dotenv")]
use dotenv::dotenv;

use {
    airkorea::Grade,
    backslash_z::*,
    daumdic::Lang,
    futures::prelude::*,
    lazy_static::lazy_static,
    openssl_probe,
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
                let mut word = w.word.clone();
                let mut body = String::new();
                if let Lang::Other(ref lang) = w.lang {
                    word.push_str(" ");
                    word.push_str(lang);
                }
                if let Some(ref pronounce) = w.pronounce {
                    body.push_str(&pronounce);
                    body.push_str(" ");
                }
                body.push_str(&w.meaning.join(", "));
                (word, body, false)
            }))
            .footer(|f| f.text("daumdic"))
        }),
        Response::AirPollution(ref status) => m.embed(|e| {
            e.description(format!("{}. {}", status.station_address, status.time))
                .fields(status.pollutants.iter().map(|p| {
                    (
                        &p.name,
                        format!(
                            "{} ({}): {}  {}",
                            p.name,
                            p.unit,
                            p.data
                                .iter()
                                .skip(p.data.len() - 5)
                                .map(|p| p
                                    .map(|f| f.to_string())
                                    .unwrap_or_else(|| "--".to_string()))
                                .collect::<Vec<_>>()
                                .join(" → "),
                            match p.grade {
                                Grade::None => "정보없음",
                                Grade::Good => "좋음",
                                Grade::Normal => "보통",
                                Grade::Bad => "나쁨",
                                Grade::Critical => "매우나쁨",
                            },
                        ),
                        true,
                    )
                }))
        }),
        Response::HowTo(ref answer) => m.embed(|e| {
            e.title(&answer.question_title)
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
            let fut = req.request(&*APP_CONFIG).then(move |resp| {
                let r = match resp {
                    Ok(resp) => msg.channel_id.send_message(|m| format_resp(m, &resp)),
                    Err(why) => {
                        eprintln!("Error while requesting: {:?}", why);
                        msg.channel_id.send_message(|m| m.content("._."))
                    }
                };
                if let Err(why) = r {
                    eprintln!("Error while sending message: {:?}", why);
                }
                Ok(())
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
    openssl_probe::init_ssl_cert_env_vars();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment variables");

    let mut client = Client::new(&token, Handler).expect("Err creating client");

    if let Err(why) = client.start() {
        eprintln!("Client error: {:?}", why);
    }
}
