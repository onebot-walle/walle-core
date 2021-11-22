use std::sync::Arc;

use clap::{App, Arg, ArgMatches, Error};
use tracing::{info, warn};
use walle_core::{Message, MessageBuild};

pub(crate) struct Cache {
    pub(crate) clap: App<'static>,
    pub(crate) user_id: String,
    pub(crate) group_id: String,
    pub(crate) cli: Arc<walle_core::app::OneBot>,
}

impl Cache {
    pub(crate) fn new(cli: Arc<walle_core::app::OneBot>) -> Self {
        Self {
            clap: build_app(),
            user_id: String::default(),
            group_id: String::default(),
            cli,
        }
    }

    pub(crate) async fn handle_input(&mut self, input: &str) {
        if input.starts_with(":") {
            match parse(input, &mut self.clap) {
                Ok(m) => self.handle_matches(m).await,
                Err(e) => e.print().unwrap(),
            }
        } else {
            if !self.user_id.is_empty() {
                self.cli
                    .send_message(
                        "private".to_owned(),
                        None,
                        Some(self.user_id.clone()),
                        Message::new().text(input.to_owned()),
                    )
                    .await;
            } else if !self.group_id.is_empty() {
                self.cli
                    .send_message(
                        "group".to_owned(),
                        Some(self.group_id.clone()),
                        None,
                        Message::new().text(input.to_owned()),
                    )
                    .await;
            } else {
                warn!("no group or user id is setted");
            }
        }
    }

    pub(crate) async fn handle_matches(&mut self, matches: ArgMatches) {
        if let Some(user_id) = matches
            .subcommand_matches("set_user_id")
            .and_then(|a| a.value_of("user_id"))
        {
            self.user_id = user_id.to_string();
            self.group_id = String::default();
            info!(target:"mira", "Set UserId {}", user_id);
            return;
        }
        if let Some(group_id) = matches
            .subcommand_matches("set_group_id")
            .and_then(|a| a.value_of("group_id"))
        {
            self.group_id = group_id.to_string();
            self.user_id = String::default();
            info!(target:"mira", "Set GroupId {}", group_id);
            return;
        }
    }
}

fn build_app() -> App<'static> {
    let app = App::new(":")
        .version("0.1.0")
        .subcommand(
            App::new("set_user_id")
                .about("set a glodal user_id ")
                .arg(Arg::new("user_id").required(true)),
        )
        .subcommand(
            App::new("set_group_id")
                .about("set a glodal group_id ")
                .arg(Arg::new("group_id").required(true)),
        );
    app
}

fn parse(input: &str, app: &mut App<'static>) -> Result<ArgMatches, Error> {
    let input = format!(": {}", input.split_at(1).1);
    app.try_get_matches_from_mut(input.split_whitespace())
}

#[test]
fn parse_test() {
    let mut app = build_app();
    match parse(": -h", &mut app) {
        Ok(matches) => println!("{:?}", matches),
        Err(e) => e.print().unwrap(), // app.print_help().unwrap(),
    }
}
