use std::io::Read;
use std::iter::FromIterator;
use std::string;
use std::vec;

use rand::SeedableRng;
use rand::seq::SliceRandom;

use lazy_static::lazy_static;

use regex::Regex;

use serenity::model::prelude::command::Command;
use serenity::prelude::*;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::framework::standard::macros::{command,group};
use serenity::framework::standard::{StandardFramework, CommandResult};
struct Handler;

struct insultsKey;
impl TypeMapKey for insultsKey{
    type Value= Vec<String>;
}

#[async_trait]
impl EventHandler for Handler{

    async fn ready(&self, _: Context, ready: Ready){
        println!("{} is connected.",ready.user.name);
    }
}

async fn user_respond(message: &Message) -> String{
    lazy_static! {
        //any message passed to function *SHOULD be garunteed to have a bot prefix/command  
        static ref re_mention: Regex = Regex::new("^(?:\\S* )(<@\\d*>)").unwrap();
    }
    let msg = message.content.as_str();
    let cap=re_mention.captures(msg);
    if cap.is_some(){
        return cap.unwrap().get(1).unwrap().as_str().to_string()
    }
    "<@".to_owned()+&message.author.id.to_string()+">"
}

#[command]
async fn doughnut(context: &Context, message: &Message) -> CommandResult{
    let mut response=user_respond(&message).await;
    response.push_str(", you fucking doughnut!");
    message.channel_id.say(&context.http,response).await?;
    Ok(())
}

#[command]
async fn donkey(context: &Context, message: &Message) -> CommandResult{
    let mut response=user_respond(&message).await;
    response.push_str(", you fucking donkey!");
    message.channel_id.say(&context.http,response).await?;
    Ok(())
}

#[command]
async fn insult(context: &Context, message: &Message) -> CommandResult{
    let mut response = user_respond(&message).await+" ";
    let data=context.data.read().await;
    let insults=data.get::<insultsKey>().expect("Access insults vector in shared context data.");
    let mut rng=rand::rngs::StdRng::from_entropy();
    let insult=insults.choose(&mut rng).expect("choose random insult.");
    response.push_str(insult);
    message.channel_id.say(&context.http,response).await?;
    Ok(())
}

#[command]
async fn help(context: &Context, message: &Message) -> CommandResult{
    let mut response = "<@".to_owned()+&message.author.id.to_string()+">";
    response.push_str(" look at you, you fucking doughnut.
Use the bot prefix **gr!** with commands.
**Commands:**
    **help** - get help with using this bot.
    **insult** - Generate a random insult. Tag someone to insult them instead!
For other insults, try:
    **doughnut**
    **donkey**
    ");
    message.channel_id.say(&context.http,response).await?;
    Ok(())
}

//unimplemented:
#[command]
async fn join(context: &Context, message: &Message) -> CommandResult{
    Ok(())
}

#[command]
async fn leave(context: &Context, message: &Message) -> CommandResult{
    Ok(())
}

#[group]
#[commands(doughnut,donkey,insult,help)]
struct General;

#[tokio::main]
async fn main(){
    //TODO remove unwrap, (panics)
    dotenv::dotenv().expect("Failed to load .env file.");
    let discord_token=dotenv::var("DISCORD_TOKEN").expect("Failed to get discord token from env.");
    println!("Discord Token: {}",discord_token);

    let insults_fpath="insults.txt";

    

    let mut str_buf=String::new();
    {    
    let mut f=std::fs::File::open(insults_fpath).expect("open insults.txt");
    f.read_to_string(&mut str_buf).expect("Failed to read insult txt file.");
    }
    
    let mut insults: Vec<String>=Vec::new();//TODO use iterator conversion instead
    for i in str_buf.lines(){
        insults.push(i.to_string());
    }

    let intents=GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let framework=StandardFramework::new().configure(|c| c.prefix("gr!")).group(&GENERAL_GROUP);
    let mut client=Client::builder(&discord_token,intents).event_handler(Handler).framework(framework).
    await.expect("Error creating client.");

    {
        let mut data=client.data.write().await;
        data.insert::<insultsKey>(insults);
    }

    if let Err(why)=client.start().await{
        println!("Client error: {}",why);
    }  
}