mod cmd;

fn main() -> miette::Result<()> {
    cmd::Cmd::default().exec()
}
