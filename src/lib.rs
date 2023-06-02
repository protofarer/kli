use anyhow::{anyhow, Context, Result};
use clap::Parser;

#[derive(Parser)]
pub struct Args {
    // #[arg(short = 'o', long = "output")]
    pattern: String,
    path: std::path::PathBuf,
}

#[derive(Debug)]
struct CustomError(String);

pub fn run() -> Result<()> {
    let args = Args::parse();

    if args.pattern.is_empty() {
        return Err(anyhow!("pattern cannot be empty"));
    }

    // ! main doesn't see the read_to_string level error
    let content = std::fs::read_to_string(&args.path)
        .with_context(|| format!("could not read file `{}`", &args.path.display()))?;

    find_matches(&content, &args.pattern, &mut std::io::stdout())?;

    Ok(())
}

pub fn find_matches(content: &str, pattern: &str, mut writer: impl std::io::Write) -> Result<()> {
    for line in content.lines() {
        if line.contains(pattern) {
            writeln!(writer, "{}", line)
                .with_context(|| format!("problem writing to buffer"))
                .unwrap();
        }
    }
    Ok(())
}

// let pb = indicatif::ProgressBar::new(100);
// pb.set_style(
//     ProgressStyle::with_template(
//         "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
//     )
//     .unwrap()
//     .progress_chars("##-"),
// );
// for i in 0..20 {
//     thread::sleep(Duration::from_millis(100));
//     pb.println(format!("[+] finished {}%", i * 5));
//     pb.inc(5);
// }
// pb.finish_with_message("Ok, DONE!");
