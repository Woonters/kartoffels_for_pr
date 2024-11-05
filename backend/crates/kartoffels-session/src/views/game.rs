mod bottom;
mod camera;
mod ctrl;
mod dialog;
mod event;
mod map;
mod perms;
mod side;

use self::bottom::*;
use self::camera::*;
pub use self::ctrl::*;
use self::dialog::*;
pub use self::dialog::{HelpMsg, HelpMsgRef, HelpMsgResponse};
use self::event::*;
use self::map::*;
pub use self::perms::*;
use self::side::*;
use anyhow::Result;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use futures_util::FutureExt;
use itertools::Either;
use kartoffels_store::{SessionId, Store};
use kartoffels_ui::{Clear, Fade, FadeDir, Render, Term, Ui};
use kartoffels_world::prelude::{
    BotId, ClockSpeed, CreateBotRequest, Handle as WorldHandle,
    Snapshot as WorldSnapshot, SnapshotStream,
};
use ratatui::layout::{Constraint, Layout};
use std::future::Future;
use std::ops::ControlFlow;
use std::sync::Arc;
use std::time::Instant;
use tokio::select;
use tracing::debug;

pub async fn run<CtrlFn, CtrlFut>(
    store: &Store,
    sess: SessionId,
    term: &mut Term,
    ctrl: CtrlFn,
) -> Result<()>
where
    CtrlFn: FnOnce(GameCtrl) -> CtrlFut,
    CtrlFut: Future<Output = Result<()>>,
{
    let (tx, rx) = GameCtrl::new();
    let view = Box::pin(run_once(store, sess, term, rx));
    let ctrl = Box::pin(ctrl(tx));

    select! {
        result = view => result,
        result = ctrl => result,
    }
}

async fn run_once(
    store: &Store,
    sess: SessionId,
    term: &mut Term,
    mut ctrl: GameCtrlRx,
) -> Result<()> {
    debug!("run()");

    let mut fade = Some(Fade::new(FadeDir::In));
    let mut tick = Instant::now();
    let mut state = State::default();

    loop {
        let event = term
            .frame(|ui| {
                state.tick(tick.elapsed().as_secs_f32(), store);
                state.render(ui, sess);

                if let Some(fade) = &fade {
                    fade.render(ui);
                }

                tick = Instant::now();
            })
            .await?;

        if let Some(event) = event {
            if let ControlFlow::Break(_) =
                event.handle(store, sess, term, &mut state).await?
            {
                fade = Some(Fade::new(FadeDir::Out));
            }
        }

        state.poll(term, &mut ctrl).await?;

        if let Some(fade) = &fade {
            if fade.dir() == FadeDir::Out && fade.is_completed() {
                return Ok(());
            }
        }
    }
}

#[derive(Default)]
struct State {
    bot: Option<JoinedBot>,
    camera: Camera,
    dialog: Option<Dialog>,
    handle: Option<WorldHandle>,
    help: Option<HelpMsgRef>,
    map: Map,
    paused: bool,
    perms: Perms,
    snapshot: Arc<WorldSnapshot>,
    snapshots: Option<SnapshotStream>,
    speed: ClockSpeed,
    status: Option<(String, Instant)>,
}

impl State {
    fn tick(&mut self, dt: f32, store: &Store) {
        if let Some(bot) = &self.bot {
            if bot.follow {
                if let Some(bot) = self.snapshot.bots().alive().by_id(bot.id) {
                    self.camera.animate_to(bot.pos);
                }
            }
        }

        self.camera.tick(dt, store);
    }

    fn render(&mut self, ui: &mut Ui<Event>, sess: SessionId) {
        let [main_area, bottom_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)])
                .areas(ui.area());

        let [map_area, side_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(SidePanel::WIDTH),
        ])
        .areas(main_area);

        Clear::render(ui);

        ui.enable(self.dialog.is_none(), |ui| {
            ui.clamp(bottom_area, |ui| {
                BottomPanel::render(ui, self);
            });

            if self.handle.is_some() {
                ui.enable(self.perms.ui_enabled, |ui| {
                    ui.clamp(side_area, |ui| {
                        SidePanel::render(ui, self);
                    });

                    ui.clamp(map_area, |ui| {
                        Map::render(ui, self);
                    });
                });
            }
        });

        if let Some(dialog) = &mut self.dialog {
            dialog.render(ui, sess, &self.snapshot);
        }
    }

    async fn poll(
        &mut self,
        term: &mut Term,
        ctrl: &mut GameCtrlRx,
    ) -> Result<()> {
        while let Some(event) = ctrl.recv().now_or_never().flatten() {
            event.handle(self, term).await?;
        }

        if let Some(snapshots) = &mut self.snapshots {
            if let Some(snapshot) = snapshots.next().now_or_never() {
                self.update_snapshot(snapshot?);
            }
        }

        Ok(())
    }

    fn update_snapshot(&mut self, snapshot: Arc<WorldSnapshot>) {
        // If map size's changed, recenter the camera - this comes handy for
        // controllers which call `world.set_map()`, e.g. the tutorial
        if snapshot.map().size() != self.snapshot.map().size() {
            self.camera.move_to(snapshot.map().center());
        }

        self.snapshot = snapshot;

        if let Some(bot) = &mut self.bot {
            let exists_now = self.snapshot.bots().by_id(bot.id).is_some();

            bot.exists |= exists_now;

            if bot.exists && !exists_now {
                self.bot = None;
            }
        }
    }

    async fn pause(&mut self) -> Result<()> {
        if !self.paused {
            self.paused = true;
            self.snapshots = None;

            if self.perms.sync_pause
                && let Some(handle) = &self.handle
            {
                handle.pause().await?;
            }
        }

        Ok(())
    }

    async fn resume(&mut self) -> Result<()> {
        if self.paused {
            self.paused = false;

            self.snapshots =
                self.handle.as_ref().map(|handle| handle.snapshots());

            if self.perms.sync_pause
                && let Some(handle) = &self.handle
            {
                handle.resume().await?;
            }
        }

        Ok(())
    }

    fn join_bot(&mut self, id: BotId) {
        self.bot = Some(JoinedBot {
            id,
            follow: true,
            exists: false,
        });

        self.map.blink = Instant::now();
    }

    async fn upload_bot(&mut self, src: Either<String, Vec<u8>>) -> Result<()> {
        let src = match src {
            Either::Left(src) => {
                let src = src.trim().replace('\r', "");
                let src = src.trim().replace('\n', "");

                match BASE64_STANDARD.decode(src) {
                    Ok(src) => src,
                    Err(err) => {
                        self.dialog = Some(Dialog::Error(ErrorDialog::new(
                            format!("couldn't decode pasted content:\n\n{err}"),
                        )));

                        return Ok(());
                    }
                }
            }

            Either::Right(src) => src,
        };

        let id = self
            .handle
            .as_ref()
            .unwrap()
            .create_bot(CreateBotRequest::new(src))
            .await;

        let id = match id {
            Ok(id) => id,

            Err(err) => {
                self.dialog =
                    Some(Dialog::Error(ErrorDialog::new(format!("{err:?}"))));

                return Ok(());
            }
        };

        self.join_bot(id);
        self.resume().await?;

        Ok(())
    }
}

#[derive(Debug)]
struct JoinedBot {
    id: BotId,
    follow: bool,
    exists: bool,
}
