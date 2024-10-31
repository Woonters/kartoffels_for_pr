use crate::TestContext;
use kartoffels_world::prelude::ClockSpeed;
use std::time::Duration;
use termwiz::input::{KeyCode, Modifiers};
use tokio::time;

async fn ctxt() -> TestContext {
    let mut ctxt = TestContext::new([]).await;

    ctxt.wait_for(TestContext::HOME).await;
    ctxt.see("[t] tutorial");
    ctxt.press(KeyCode::Char('t')).await;

    ctxt.wait_for("ready?").await;
    ctxt.see("[esc] no, leave tutorial");
    ctxt.see("[enter] yes, start tutorial");
    ctxt
}

#[tokio::test]
async fn smoke() {
    let mut ctxt = ctxt().await;

    ctxt.see_frame("tutorial/smoke/1.txt").await;
}

#[tokio::test]
async fn leave() {
    let mut ctxt = ctxt().await;

    ctxt.dont_see(TestContext::HOME);
    ctxt.press(KeyCode::Escape).await;
    ctxt.wait_for(TestContext::HOME).await;
}

#[tokio::test]
async fn leave_and_start() {
    let mut ctxt = ctxt().await;

    ctxt.dont_see(TestContext::HOME);
    ctxt.press(KeyCode::Escape).await;
    ctxt.wait_for(TestContext::HOME).await;

    ctxt.press(KeyCode::Char('t')).await;
    ctxt.wait_for("ready?").await;
    ctxt.see("[esc] no, leave tutorial");
    ctxt.see("[enter] yes, start tutorial");
}

#[tokio::test]
async fn leave_using_ctrl_c() {
    let mut ctxt = ctxt().await;

    ctxt.dont_see(TestContext::HOME);
    ctxt.press_ex(KeyCode::Char('a'), Modifiers::CTRL).await;
    ctxt.wait_for(TestContext::HOME).await;
}

#[tokio::test]
async fn flow() {
    let mut ctxt = ctxt().await;

    ctxt.see_frame("tutorial/flow/step-01.txt").await;

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for_modal("tutorial (2/16)").await;
    ctxt.see_frame("tutorial/flow/step-02.txt").await;

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for_modal("tutorial (3/16)").await;
    ctxt.see_frame("tutorial/flow/step-03.txt").await;

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for_modal("tutorial (4/16)").await;
    ctxt.see_frame("tutorial/flow/step-04.txt").await;

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for_modal("tutorial (5/16)").await;
    ctxt.see_frame("tutorial/flow/step-05.txt").await;

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_while_modal("tutorial (5/16)").await;
    ctxt.see_frame("tutorial/flow/step-05-a.txt").await;

    ctxt.press(KeyCode::Char('h')).await;
    ctxt.wait_for_modal("help").await;
    ctxt.see_frame("tutorial/flow/step-05-b.txt").await;

    ctxt.press(KeyCode::Escape).await;
    ctxt.wait_while_modal("help").await;
    ctxt.see_frame("tutorial/flow/step-05-c.txt").await;

    // ---

    ctxt.upload_bot("tut-01").await;
    ctxt.wait_for_modal("tutorial (6/16)").await;
    ctxt.see_frame("tutorial/flow/step-06.txt").await;

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for_modal("tutorial (7/16)").await;
    ctxt.see_frame("tutorial/flow/step-07-a.txt").await;

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for("alive (0s)").await;
    ctxt.see_frame("tutorial/flow/step-07-b.txt").await;

    time::pause();
    time::advance(Duration::from_secs(10)).await;
    time::resume();

    // ---

    ctxt.wait_for_modal("tutorial (8/16)").await;
    ctxt.see_frame("tutorial/flow/step-08.txt").await;

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for_modal("tutorial (9/16)").await;
    ctxt.see_frame("tutorial/flow/step-09-a.txt").await;

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_while_modal("tutorial (9/16)").await;
    ctxt.see_frame("tutorial/flow/step-09-b.txt").await;

    ctxt.press(KeyCode::Char('h')).await;
    ctxt.wait_for_modal("help").await;
    ctxt.see_frame("tutorial/flow/step-09-c.txt").await;

    ctxt.press(KeyCode::Escape).await;
    ctxt.wait_while_modal("help").await;
    ctxt.see_frame("tutorial/flow/step-09-d.txt").await;

    // ---

    ctxt.upload_bot("tut-01").await;
    ctxt.wait_for_modal("tutorial (10/16)").await;
    ctxt.see_frame("tutorial/flow/step-10-a.txt").await;

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_while_modal("tutorial (10/16)").await;

    time::pause();
    time::advance(Duration::from_secs(15)).await;
    time::resume();

    ctxt.wait_for_modal("tutorial (10/16)").await;
    ctxt.see_frame("tutorial/flow/step-10-b.txt").await;

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_while_modal("tutorial (10/16)").await;
    ctxt.wait_while("0543-c377-57f0-8d9a").await;
    ctxt.see_frame("tutorial/flow/step-10-c.txt").await;

    ctxt.upload_bot("tut-02").await;

    ctxt.store()
        .first_private_world()
        .overclock(ClockSpeed::Fastest)
        .await
        .unwrap();

    // ---

    ctxt.wait_for_modal("tutorial (11/16)").await;
    ctxt.see_frame("tutorial/flow/step-11.txt").await;

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for_modal("tutorial (12/16)").await;
    ctxt.see_frame("tutorial/flow/step-12.txt").await;

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for_modal("tutorial (13/16)").await;
    ctxt.see_frame("tutorial/flow/step-13-a.txt").await;

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_while_modal("tutorial (13/16)").await;
    ctxt.wait_for("......... .........").await;
    ctxt.see_frame("tutorial/flow/step-13-b.txt").await;

    ctxt.press(KeyCode::Char('h')).await;
    ctxt.wait_for_modal("help").await;
    ctxt.see_frame("tutorial/flow/step-13-c.txt").await;

    ctxt.press(KeyCode::Escape).await;
    ctxt.wait_while_modal("help").await;
    ctxt.see_frame("tutorial/flow/step-13-d.txt").await;

    ctxt.upload_bot("tut-02").await;
    ctxt.wait_for_modal("tutorial (13/16)").await;
    ctxt.see_frame("tutorial/flow/step-13-e.txt").await;

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_while_modal("tutorial (13/16)").await;
    ctxt.see_frame("tutorial/flow/step-13-f.txt").await;

    ctxt.upload_bot("tut-03").await;

    ctxt.store()
        .first_private_world()
        .overclock(ClockSpeed::Unlimited)
        .await
        .unwrap();

    // ---

    ctxt.wait_for_modal("tutorial (14/16)").await;
    ctxt.see_frame("tutorial/flow/step-14.txt").await;

    // ---

    ctxt.press(KeyCode::Enter).await;
    ctxt.wait_for_modal("tutorial (15/16)").await;
    ctxt.see_frame("tutorial/flow/step-15.txt").await;

    ctxt.press(KeyCode::Enter).await;

    let mut snapshots = ctxt.store().first_private_world().snapshots();

    loop {
        let snapshot = snapshots.next().await.unwrap();

        if snapshot.bots().alive().len() == 10 {
            break;
        }
    }

    ctxt.upload_bot("tut-04").await;
    ctxt.wait_for_modal("tutorial (16/16)").await;
}
