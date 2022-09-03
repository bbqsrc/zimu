use std::{io::BufReader, os::unix::prelude::OsStrExt, path::{Path, PathBuf}};

use chrono::{NaiveTime, Timelike};
use clap::Parser;
use indexmap::IndexMap;
use zimu_ast::Block;

fn parse_file<P: AsRef<Path>>(path: P) -> Vec<Block> {
    let path = path.as_ref();
    let f = std::fs::File::open(path).unwrap();
    let f = BufReader::new(f);

    match path.extension().map(|x| x.as_bytes()) {
        Some(x) => match x {
            b"ass" | b"ssa" => ass_parser::parse(f)
                .events
                .into_iter()
                .map(zimu_ast::Block::from)
                .collect::<Vec<_>>(),
            b"srt" => srt_parser::parse(f)
                .into_iter()
                .map(zimu_ast::Block::from)
                .collect::<Vec<_>>(),
            _ => panic!("NO"),
        },
        None => todo!(),
    }
}

/// Merge multiple subtitles files together.
/// 
/// Accepts .ssa, .ass and .srt; and outputs .ass.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to main subtitles file.
   #[clap(value_parser)]
   main_subs_path: PathBuf,

   /// Path to supplementary subtitles file.
   #[clap(value_parser)]
   supplementary_subs_path: PathBuf,

   /// Main language code (e.g. zh, fr, sv)
   #[clap(short = 'm', long, value_parser)]
   main_language: String,

   /// Supplementary language code (e.g. zh, fr, sv)
   #[clap(short = 's', long, value_parser)]
   supplementary_language: String,

   /// Output path for .ass file. (Default: stdout)
   #[clap(short, long, value_parser)]
   output_path: Option<PathBuf>,   
}

fn main() {
    let args = Args::parse();
    
    let main_subs = parse_file(args.main_subs_path);
    let mut supplementary_subs = parse_file(args.supplementary_subs_path);

    normalize(main_subs.first().unwrap().start.clone(), &mut supplementary_subs);

    let supplementary_lang = args.supplementary_language;
    let main_lang = args.main_language;
    
    let mut inputs = IndexMap::new();
    inputs.insert(supplementary_lang, supplementary_subs);
    inputs.insert(main_lang, main_subs);

    let output = generate_ass(inputs);
    if let Some(output_path) = args.output_path {
        std::fs::write(output_path, output).unwrap();
    } else {
        println!("{}", output);
    }
}

fn normalize(real_start: NaiveTime, blocks: &mut Vec<zimu_ast::Block>) {
    let first_block = blocks.first().unwrap();
    let diff = real_start - first_block.start;

    for x in blocks {
        x.start += diff;
        x.end += diff;
    }
}

fn generate_ass(inputs: IndexMap<String, Vec<Block>>) -> String {
    let mut script = "[Script Info]
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
".to_string();

    for (n, lang) in inputs.keys().enumerate() {
        let x = 12 + n * (18 * 2) + n * 4;
        let font = if lang == "zh" {
            "Microsoft YaHei"
        } else {
            "Arial"
        };
        // This is BGR, not RGB.
        let color = if n == 0 { "FFFFFF" } else { "00FFFF" };
        let style = format!("Style: {lang},{font},18,&H00{color},&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,1,0.5,2,10,10,{x},1\n");

        script.push_str(&style);
    }
    script.push_str("\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

    let mut n = 0usize;
    for (lang, blocks) in inputs {
        for block in blocks {
            let start = block.start.format("%k:%M:%S").to_string();
            let start = format!(
                "{}.{:02}",
                start.trim(),
                block.start.nanosecond() / 10000000
            );
            let end = block.end.format("%k:%M:%S").to_string();
            let end = format!("{}.{:02}", end.trim(), block.end.nanosecond() / 10000000);
            let text = block.content.join("\\N");
            script.push_str(&format!(
                "Dialogue: {n},{start},{end},{lang},,0,0,0,,{text}\n"
            ));
        }

        n += 1;
    }

    script
}
