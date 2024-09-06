mod step;
mod steps;

use self::step::*;
use self::steps::*;
use crate::DrivenGame;
use anyhow::Result;
use kartoffels_store::Store;

pub async fn run(store: &Store, game: DrivenGame) -> Result<()> {
    let mut ctxt = StepCtxt {
        store,
        game,
        world: None,
    };

    if !step01::run(&mut ctxt).await? {
        return Ok(());
    }

    step02::run(&mut ctxt).await?;
    step03::run(&mut ctxt).await?;
    step04::run(&mut ctxt).await?;
    step05::run(&mut ctxt).await?;
    step06::run(&mut ctxt).await?;
    step07::run(&mut ctxt).await?;
    step08::run(&mut ctxt).await?;
    step09::run(&mut ctxt).await?;
    step10::run(&mut ctxt).await?;

    Ok(())
}
