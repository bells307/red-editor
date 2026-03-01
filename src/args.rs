use std::{env, path::PathBuf};

pub struct Args {
    pub file: Option<PathBuf>,
}

impl Args {
    pub fn parse() -> Args {
        let file = env::args().nth(1).map(PathBuf::from);
        Args { file }
    }
}
