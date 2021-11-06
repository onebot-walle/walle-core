use std::sync::Arc;

use clap::{App, Arg, ArgMatches, Error};

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

    pub(crate) fn handle_input(&mut self, input: String) {
        if input.starts_with(":") {
            match parse(input, &mut self.clap) {
                Ok(m) => {}
                Err(e) => e.print().unwrap(),
            }
        }
    }

    pub(crate) fn handle_matches(&mut self, matches: ArgMatches) {
        if let Some(user_id) = matches.value_of("user_id") {
            self.user_id = user_id.to_string();
        }
        if let Some(group_id) = matches.value_of("group_id") {
            self.group_id = group_id.to_string();
        }
    }
}

fn build_app() -> App<'static> {
    let app = App::new(":")
        .version("0.1.0")
        .subcommand(
            App::new("send")
                .about("send the message")
                .arg(Arg::new("message").required(true)),
        )
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

fn parse(mut input: String, app: &mut App<'static>) -> Result<ArgMatches, Error> {
    input.insert(1, ' ');
    app.try_get_matches_from_mut(input.split_whitespace())
}

#[test]
fn parse_test() {
    let mut app = build_app();
    match parse(": -h".to_owned(), &mut app) {
        Ok(matches) => println!("{:?}", matches),
        Err(e) => e.print().unwrap(), // app.print_help().unwrap(),
    }
}
