use std::sync::Arc;

use clap::{App, Arg, ArgMatches, Error};
use colored::*;
use tracing::info;

pub(crate) struct Cache {
    pub(crate) clap: App<'static>,
    pub(crate) user_id: String,
    pub(crate) group_id: String,
    pub(crate) cli: Arc<walle_core::app::StandardOneBot>,
}

impl Cache {
    pub(crate) fn new(cli: Arc<walle_core::app::StandardOneBot>) -> Self {
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
            self.send_message(input).await;
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
        } else if let Some(group_id) = matches
            .subcommand_matches("set_group_id")
            .and_then(|a| a.value_of("group_id"))
        {
            self.group_id = group_id.to_string();
            self.user_id = String::default();
            info!(target:"mira", "Set GroupId {}", group_id);
            return;
        } else if let Some(_) = matches.subcommand_matches("bots") {
            let bots = self.cli.get_bots().await;
            for (id, _bot) in bots.iter() {
                info!(target:"mira", "Bot {}", id);
            }
        } else if let Some(_) = matches.subcommand_matches("get_status") {
            for (id, bot) in self.cli.get_bots().await.iter() {
                info!(target:"mira", "[{}]{:?}", id.red(), bot.get_status().await);
            }
        } else if let Some(_) = matches.subcommand_matches("get_version") {
            for (id, bot) in self.cli.get_bots().await {
                info!(target: "mira", "[{}]{:?}", id, bot.get_version().await);
            }
        } else if let Some(_) = matches.subcommand_matches("get_supported_actions") {
            for (id, bot) in self.cli.get_bots().await {
                info!(target: "mira", "[{}]{:?}", id, bot.get_supported_actions().await);
            }
        } else if let Some(_) = matches.subcommand_matches("get_self_info") {
            for (id, bot) in self.cli.get_bots().await {
                info!(target: "mira", "[{}]{:?}", id, bot.get_self_info().await);
            }
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
        )
        .subcommand(App::new("get_status"))
        .subcommand(App::new("get_version"))
        .subcommand(App::new("get_supported_actions"))
        .subcommand(App::new("get_self_info"))
        .subcommand(App::new("bots").about("show connectted bot"));
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
