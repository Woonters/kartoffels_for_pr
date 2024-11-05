use super::prelude::*;

static MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" tutorial (11/16) "),

    body: vec![
        MsgLine::new("nice!"),
        MsgLine::new(""),
        MsgLine::new("i mean, not nice, because we're dead, but baby steps"),
        MsgLine::new(""),
        MsgLine::new(
            "now it's time for you to learn about *da radar*, using which you \
             can program the bot to avoid falling out the map",
        ),
    ],

    buttons: vec![MsgButton::confirm("let's learn", ())],
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    ctxt.game.show_msg(&MSG).await?;

    Ok(())
}
