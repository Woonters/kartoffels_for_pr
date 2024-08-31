mod header;
mod menu;

use self::header::*;
use self::menu::*;
use crate::{theme, Clear, Term};
use anyhow::Result;
use ratatui::layout::{Constraint, Layout};
use tokio::time;

pub async fn run(term: &mut Term) -> Result<Response> {
    loop {
        let mut response = None;

        term.draw(|ui| {
            let [_, area, _] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(Header::WIDTH),
                Constraint::Fill(1),
            ])
            .areas(ui.area());

            let [_, header_area, _, menu_area, _, _] = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(Header::HEIGHT),
                Constraint::Length(1),
                Constraint::Length(Menu::HEIGHT),
                Constraint::Length(1),
                Constraint::Fill(2),
            ])
            .areas(area);

            Clear::render(ui);

            ui.clamp(header_area, |ui| {
                Header::render(ui);
            });

            ui.clamp(menu_area, |ui| {
                response = Menu::render(ui);
            });
        })
        .await?;

        if let Some(response) = response {
            time::sleep(theme::INTERACTION_TIME).await;

            return Ok(response);
        }

        term.tick().await?;
    }
}

#[derive(Debug)]
pub enum Response {
    Play,
    OpenTutorial,
    OpenChallenges,
    Quit,
}
