use super::SidePanelResponse;
use crate::views::play::JoinedBot;
use crate::{theme, BotIdExt, Button, RectExt, Ui};
use itertools::Either;
use kartoffels_world::prelude::Snapshot;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct JoinedSidePanel;

impl JoinedSidePanel {
    const FOOTER_HEIGHT: u16 = 3;

    pub fn render(
        ui: &mut Ui,
        world: &Snapshot,
        bot: &JoinedBot,
        enabled: bool,
    ) -> Option<SidePanelResponse> {
        let mut response = None;

        ui.line("id".underlined());
        ui.line(bot.id.to_string().fg(bot.id.color()));
        ui.space(1);

        match world.bots.by_id(bot.id) {
            Some(Either::Left(bot)) => {
                ui.line("status".underlined());
                ui.line(Line::from_iter([
                    "alive ".fg(theme::GREEN),
                    format!("({}s)", bot.age).into(),
                ]));
                ui.space(1);

                // ---

                let area = {
                    let area = ui.area();

                    Rect {
                        x: area.x,
                        y: area.y,
                        width: area.width,
                        height: area.height - Self::FOOTER_HEIGHT - 1,
                    }
                };

                ui.clamp(area, |ui| {
                    ui.line("serial port".underlined());

                    Paragraph::new(bot.serial.as_str())
                        .wrap(Default::default())
                        .render(ui.area(), ui.buf());
                });
            }

            Some(Either::Right(bot)) => {
                ui.line("status".underlined());

                ui.row(|ui| {
                    let status = if bot.requeued {
                        "killed, requeued"
                    } else {
                        "killed, queued"
                    };

                    ui.span(status.fg(theme::PINK));
                    ui.span(format!(" ({})", bot.place));
                });
            }

            None => {
                ui.line("status".underlined());
                ui.line("killed, dead".fg(theme::RED));
            }
        }

        ui.clamp(ui.area().footer(Self::FOOTER_HEIGHT), |ui| {
            if Button::new(KeyCode::Char('f'), "follow")
                .enabled(enabled)
                .block()
                .render(ui)
                .pressed
            {
                todo!();
            }

            if Button::new(KeyCode::Char('h'), "history")
                .enabled(enabled)
                .block()
                .render(ui)
                .pressed
            {
                response = Some(SidePanelResponse::ShowBotHistory);
            }

            if Button::new(KeyCode::Char('l'), "leave bot")
                .enabled(enabled)
                .block()
                .render(ui)
                .pressed
            {
                response = Some(SidePanelResponse::LeaveBot);
            }
        });

        response
    }
}
