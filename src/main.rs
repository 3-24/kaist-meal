use std::env;

use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::model::interactions::{Interaction, InteractionResponseType};
use serenity::prelude::*;

use dotenv::dotenv;
use scraper::{Html, Selector};

use chrono::prelude::*;

struct Handler;

enum MealTime {
    Breakfast,
    Lunch,
    Dinner,
}

async fn parse_meal(location: &str, meal_time: MealTime) -> String {
    let loc_get_param = match location {
        "카이마루" => "fclt",
        "교수회관" => "emp",
        _ => {
            panic!()
        }
    };
    let query_url = format!(
        "https://www.kaist.ac.kr/kr/html/campus/053001.html?dvs_cd={}",
        loc_get_param
    );
    let resp = reqwest::get(query_url)
        .await
        .expect("await fail")
        .text()
        .await
        .expect("await fail");
    let parsed_html = Html::parse_document(&resp);

    let css_selector = match meal_time {
        MealTime::Breakfast => "#tab_item_1 > table > tbody > tr > td:nth-child(1) > ul",
        MealTime::Lunch => "#tab_item_1 > table > tbody > tr > td:nth-child(2) > ul",
        MealTime::Dinner => "#tab_item_1 > table > tbody > tr > td:nth-child(3) > ul",
    };
    let selector = &Selector::parse(css_selector).expect("Error during the paring using selector");
    let parsed_result = parsed_html
        .select(selector)
        .next()
        .expect("Error during selection")
        .inner_html()
        .replace("<br>", "")
        .replace("&amp", "&");

    parsed_result
}

fn get_current_time() -> MealTime {
    let local = Utc::now()
        .with_timezone(&FixedOffset::east(3600 * 9))
        .time(); // KST

    let meal_cut = NaiveTime::from_hms(13, 30, 0);
    if local > meal_cut {
        MealTime::Dinner
    } else {
        MealTime::Lunch
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let command_name = command.data.name.as_str();
            let content = match command_name {
                "카이마루" | "교수회관" => {
                    let result: String = parse_meal(command_name, get_current_time()).await;
                    result
                }
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| command.name("카이마루").description("밥"))
                .create_application_command(|command| command.name("교수회관").description("바압"))
        })
        .await;

        println!(
            "I now have the following guild slash commands: {:#?}",
            commands
        );
        /*
        let guild_command =
            ApplicationCommand::create_global_application_command(&ctx.http, |command| {
                command.name("wonderful_command").description("An amazing command")
            })
            .await;

        println!("I created the following global slash command: {:#?}", guild_command);
        */
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Build our client.
    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
