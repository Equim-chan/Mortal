use std::env;

use anyhow::{Context, Result};
use riichi::stat::Stat;

const USAGE: &str = "Usage: stat <DIR> <PLAYER_NAME>";

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    let dir = args.get(1).context(USAGE)?;
    let player_name = args.get(2).context(USAGE)?;

    let stat = Stat::from_dir(dir, player_name, false)?;
    println!("{stat}");

    Ok(())
}
