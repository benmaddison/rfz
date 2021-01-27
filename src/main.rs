use rfz::{Cli, Defaults, Result};

fn main() -> Result<()> {
    let defaults = Defaults::get()?;
    let cli = Cli::init(&defaults);
    cli.run()
}
