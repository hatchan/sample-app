use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::error;

use crate::{
    client::{Client, ClientError},
    models::{CreateUserError, GetUserError, NewUser, User},
};

#[derive(Parser)]
pub struct Args {
    #[clap(
        short,
        long,
        env,
        default_value = "http://127.0.0.1:3000",
        global = true
    )]
    endpoint: url::Url,

    #[command(subcommand)]
    command: SubCommand,
}

#[derive(Subcommand)]
pub enum SubCommand {
    Get(GetArgs),
    Create(CreateArgs),
}

pub async fn handle_command(args: Args) -> Result<()> {
    match args.command {
        SubCommand::Get(args) => handle_get(args).await,
        SubCommand::Create(args) => handle_create(args).await,
    }
}

#[derive(Parser)]
pub struct GetArgs {
    pub username: String,

    #[clap(from_global)]
    pub endpoint: url::Url,
}

async fn handle_get(args: GetArgs) -> Result<()> {
    let client = Client::new(args.endpoint);
    match client.get_user(&args.username).await {
        Ok(user) => println!("{:#?}", user),
        Err(err) => match err {
            ClientError::ConnectionError => error!("Connection error"),
            ClientError::TimeoutError => error!("Timeout occurred"),
            ClientError::UnknownError => error!("Unknown error"),
            ClientError::DeserializationError => {
                error!("Unable to deserialize response")
            }
            ClientError::Unauthenticated => error!("Unauthenticated"),
            ClientError::Unauthorized => error!("Unauthorized"),
            ClientError::ServiceError(err) => match err {
                GetUserError::UserNotFound { username } => {
                    error!(%username, "User not found");
                }
            },
        },
    };

    Ok(())
}

#[derive(Parser)]
pub struct CreateArgs {
    pub username: String,
    pub name: String,

    #[clap(from_global)]
    pub endpoint: url::Url,
}

async fn handle_create(args: CreateArgs) -> Result<()> {
    let user = NewUser {
        username: args.username,
        name: args.name,
    };
    let client = Client::new(args.endpoint);
    match client.create_user(user).await {
        Ok(user) => println!("{:#?}", user),
        Err(err) => match err {
            ClientError::ConnectionError => error!("Connection error"),
            ClientError::TimeoutError => error!("Timeout occurred"),
            ClientError::UnknownError => error!("Unknown error"),
            ClientError::DeserializationError => {
                error!("Unable to deserialize response")
            }
            ClientError::Unauthenticated => error!("Unauthenticated"),
            ClientError::Unauthorized => error!("Unauthorized"),
            ClientError::ServiceError(err) => match err {
                CreateUserError::UsernameAlreadyExists => {
                    error!("Username already exists")
                }
                CreateUserError::InvalidUsername(reason) => {
                    error!("Invalid username: {:?}", reason)
                }
                CreateUserError::InvalidName(reason) => error!("Invalid name: {:?}", reason),
            },
        },
    };

    Ok(())
}
