use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufWriter, BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use regex::{Regex, RegexSet};
use serde::{Serialize, Deserialize};

struct ReplacePattern {
    regex: Regex,
    replacement: String,
}

struct Patterns {
    remove: RegexSet,
    replace: Vec<ReplacePattern>,
}

fn parse_patterns<P>(path: P) -> Result<Patterns, String> where P: AsRef<Path> {
    let file = File::open(path).map_err(|e| format!("{}", e))?;
    let reader = BufReader::new(file);

    #[derive(Serialize, Deserialize)]
    struct StringReplacePattern {
        regex: String,
        replacement: String,
    }

    #[derive(Serialize, Deserialize)]
    struct StringPatterns {
        remove: Vec<String>,
        replace: Vec<StringReplacePattern>,
    }

    let StringPatterns { remove, replace } = serde_yaml::from_reader(reader)
        .map_err(|e| format!("{}", e))?;

    let remove = RegexSet::new(remove).unwrap();

    let replace = replace
        .into_iter()
        .map(|StringReplacePattern { regex, replacement }| {
            let regex = Regex::new(&regex).unwrap();
            ReplacePattern { regex, replacement }
        })
        .collect();

    Ok(Patterns { remove, replace })
}


#[derive(Parser, Debug)]
#[clap(name = "Clean")]
#[clap(author = "Aiko Wessels <aiko.wessels@gmail.com>")]
#[clap(version = "1.0")]
#[clap(about = "Cleans datasets.")]
struct Cli {
    /// Path to the patterns YAML file.
    #[clap(short, long, default_value_t = String::from("patterns.yaml"))]
    patterns: String,

    /// Comma separated list of languages to clean.
    #[clap(short, long, default_value_t = String::from("en,de,fr,es,it,pt"))]
    languages: String,

    /// Input directory, where the raw corpora are located
    #[clap(short, long, default_value_t = String::from("raw"))]
    input_dir: String,

    /// Output directory, where the processed corpora will be written to.
    #[clap(short, long, default_value_t = String::from("prepro"))]
    output_dir: String,

    /// The name of the corpus, without the language extension.
    #[clap(short, long)]
    corpus: String,
}

fn main() {
    let cli = Cli::parse();
    let corpus = cli.corpus;
    let languages:Vec<_> = cli.languages.split(',').collect();
    let input = PathBuf::from(cli.input_dir);
    let output = PathBuf::from(cli.output_dir);
    create_dir_all(&output).unwrap();

    // Read patterns from YAML file.
    let patterns_path = PathBuf::from(cli.patterns);
    let Patterns { remove, replace } = parse_patterns(patterns_path).unwrap();
    let remove = Arc::new(remove);
    let replace = Arc::new(replace);

    // Progress bar
    let mpbar = MultiProgress::new();
    let style = ProgressStyle::default_spinner()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
        .template("{prefix:.bold.dim} {spinner} {wide_msg}");

    let mut handles = Vec::new();

    for language in languages {
        let input = input.join(format!("{}.{}", corpus, language));
        let output = output.join(format!("{}.{}", corpus, language));
        let remove = remove.clone();
        let replace = replace.clone();

        let pbar = mpbar.add(ProgressBar::new_spinner());
        pbar.enable_steady_tick(100);
        pbar.set_style(style.clone());

        let handle = thread::spawn(move || {
            let in_file = File::open(&input).expect("Unable to open file");
            let reader = BufReader::new(in_file);
            let out_file = File::create(&output).expect("Unable to open file");
            let mut writer = BufWriter::new(out_file);

            // Initialize progress bar
            pbar.set_message(format!("Processing {}", input.display()));


            // Count the number of lines in the file, and the number of lines skipped.
            let mut lines_skipped = 0;
            let mut lines_total = 0;

            for line in reader.lines() {
                lines_total += 1;
                // pbar.inc(1);

                // filter all the lines that need to be removed.
                let line = line.unwrap();
                if remove.is_match(&line) {
                    lines_skipped += 1;
                    continue;
                }

                // Perform substitutions on all the tuples in replace.
                let line = replace
                    .iter()
                    .fold(line, |line, ReplacePattern { regex, replacement }| {
                        regex.replace(&line, replacement).to_string()
                    });

                writeln!(writer, "{}", line).unwrap();
            }

            // Calculate the number of lines added to the output file.
            let lines_remaining = lines_total - lines_skipped;

            // Progress bar finished.
            pbar.finish_with_message(format!("Finished processing {}, ({} / {} = {:.2})",
                input.display(),
                lines_remaining, lines_total, (lines_remaining as f32 / lines_total as f32) * 100.0
            ));
        });
        handles.push(handle);
    }

    // Wait for all progress bars to finish.
    mpbar.join().unwrap();
}
