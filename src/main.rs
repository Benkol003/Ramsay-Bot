use std::{env, fs};
use std::{
    io::Read,
    iter::FromIterator,
    string,
    vec
};

use rand::SeedableRng;
use rand::seq::{SliceRandom, IteratorRandom};

use lazy_static::lazy_static;

use regex::Regex;

type Value= Vec<String>;

use serenity::builder::CreateMessage;
use serenity::model::prelude::command::Command;
use serenity::prelude::*;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::framework::standard::macros::{command,group};
use serenity::framework::standard::{StandardFramework, CommandResult};

use songbird::input::Input;
use songbird::{SerenityInit, input, ffmpeg};

use serenity::utils::{read_image, MessageBuilder};

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
    **spaghetti_code**
    ");
    message.channel_id.say(&context.http,response).await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn join(context: &Context, message: &Message) -> CommandResult{
    let guild_id=message.guild(&context.cache).unwrap().id;
    let channel_id=message.guild(&context.cache).unwrap().voice_states.get(&message.author.id).and_then(|voice_state| voice_state.channel_id);
    if channel_id==None{
        message.reply(context,"You're not in a voice channel, you donkey.").await;
        return Ok(())
    }
    let manager = songbird::get(context).await.expect("Initialised songbird voice client").clone();
    manager.join(guild_id,channel_id.unwrap()).await;
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn leave(context: &Context, message: &Message) -> CommandResult{
    let guild_id = message.guild(&context.cache).unwrap().id;
    let manager = songbird::get(context).await.expect("Initialised songbird voice client.").clone();
    let handler = manager.get(guild_id).is_some();
    if handler{
        if let Err(err) = manager.remove(guild_id).await{
            message.reply(context, format!("Fucks sake, i cant leave: {:?}",err)).await;
        }
    }else{
        message.reply(context,"You doughnut, i'm not even here.").await;
    }
    Ok(())
}

#[command]
async fn insult(context: &Context, message: &Message) -> CommandResult{

    let AUDIO_FOLDER = "audio_clips\\";


    let guild_id = message.guild(&context.cache).unwrap().id;
    let manager = songbird::get(context).await.expect("Initialised songbird voice client.").clone();
    let mut rng=rand::rngs::StdRng::from_entropy();
    if let Some(handler) = manager.get(guild_id){

        let mut audio_folder_path = env::current_dir().unwrap();
        println!("{:?}",env::current_dir().unwrap());
        audio_folder_path.push(AUDIO_FOLDER);
        println!("EXE Path: {}",audio_folder_path.to_str().unwrap());
        let src_path = fs::read_dir(audio_folder_path).unwrap().choose(&mut rng).unwrap().unwrap().path();
        println!("audio src: {:?}",src_path.to_str());
        let mut audio_src= ffmpeg(src_path).await.expect("open file as audio.");

        let track_handler = handler.lock().await.play_only_source(audio_src);
    } else{
        let mut response = user_respond(&message).await+" ";
        let data=context.data.read().await;
        let insults=data.get::<insultsKey>().expect("Access insults vector in shared context data.");
        let insult=insults.choose(&mut rng).expect("choose random insult.");
        response.push_str(insult);
        message.channel_id.say(&context.http,response).await?;
    }

    Ok(())
}

#[command]
async fn spaghetti(context: &Context, message: &Message) -> CommandResult{
    let mut response ="**".to_string();
    response.push_str(&user_respond(&message).await);
    response.push_str(" IS NOW A DISH LOLOLOL**");
    let mut message_send=CreateMessage::default();

    message.channel_id.send_message(&context.http,|m| {
        m.content(&response).add_file("./spaghetti.jpg")
    }
    ).await?;
    Ok(())
}

#[group]
#[commands(doughnut,donkey,insult,help,join,leave, spaghetti)]
struct General;

#[tokio::main]
async fn main(){
    dotenv::dotenv().expect("Failed to load .env file.");
    let discord_token=dotenv::var("DISCORD_TOKEN").expect("Failed to get discord token from env.");
    println!("Discord Token: {}",discord_token);

    let insults_fpath=".\\insults.txt";

    

    let mut str_buf=String::new();
    {    
    let mut f=std::fs::File::open(insults_fpath).expect("open insults.txt");
    f.read_to_string(&mut str_buf).expect("Failed to read insult txt file.");
    }
    
    let mut insults: Vec<String>=Vec::new();//TODO use iterator conversion instead
    for i in str_buf.lines(){
        insults.push(i.to_string());
    }

    let intents= GatewayIntents::non_privileged() | GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let framework=StandardFramework::new().configure(|c| c.prefix("gr!")).group(&GENERAL_GROUP);
    let mut client=Client::builder(&discord_token,intents).event_handler(Handler).framework(framework).register_songbird().
    await.expect("Error creating client.");

    {
        let mut data=client.data.write().await;
        data.insert::<insultsKey>(insults);
    }


    tokio::spawn(async move{
        let _ = client.start().await.map_err(|why| println!("Client error: {}",why));
    });
    println!("ok");
    tokio::signal::ctrl_c().await;
    println!("Shutdown signal recieved.");
    //TODO make leave on shutdown
}
