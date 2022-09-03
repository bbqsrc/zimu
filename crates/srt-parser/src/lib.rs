use std::io::BufRead;

use chrono::NaiveTime;

impl From<Block> for zimu_ast::Block {
    fn from(x: Block) -> Self {
        Self {
            start: x.start,
            end: x.end,
            content: x.content,
        }
    }
} 

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    sequence_id: u64,
    start: NaiveTime,
    end: NaiveTime,
    content: Vec<String>,
    extra: Vec<String>,
}

impl Ord for Block {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for Block {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.start.partial_cmp(&other.start) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.end.partial_cmp(&other.end) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }

        None
    }
}

enum State {
    ExpectSequenceId,
    ExpectDuration,
    ExpectContentOrEnd,
}

pub fn parse<R: BufRead>(input: R) -> Vec<Block> {
    let mut blocks = vec![];
    let mut state = State::ExpectSequenceId;

    let mut cur_sequence_id: Option<u64> = None;
    let mut cur_start: Option<NaiveTime> = None;
    let mut cur_end: Option<NaiveTime> = None;
    let mut cur_extra: Vec<String> = vec![];
    let mut cur_content: Vec<String> = vec![];

    for (n, line) in input.lines().enumerate() {
        let mut line = line.expect("line was invalid");

        if line.starts_with("\u{feff}") {
            line.remove(0);
        }

        match state {
            State::ExpectSequenceId => {
                cur_sequence_id = Some(line.parse::<u64>().expect(&format!("invalid sequence id, got: {:?}", &line)));
                state = State::ExpectDuration;
            }
            State::ExpectDuration => {
                let mut chunks = line.split_whitespace();
                let start = chunks.next().expect("missing start time");
                let start = NaiveTime::parse_from_str(
                    start,
                    "%H:%M:%S,%3f",
                )
                .expect("invalid start time");
                let arrow = chunks.next().expect("missing arrow");
                if arrow != "-->" {
                    panic!("invalid item in arrow position");
                }
                let end = NaiveTime::parse_from_str(
                    chunks.next().expect("missing end time"),
                    "%H:%M:%S,%3f",
                )
                .expect("invalid end time");
                let extra = chunks.map(|x| x.to_string()).collect::<Vec<_>>();

                cur_start = Some(start);
                cur_end = Some(end);
                cur_extra = extra;

                state = State::ExpectContentOrEnd;
            }
            State::ExpectContentOrEnd => {
                if line.trim() == "" {
                    blocks.push(Block {
                        sequence_id: cur_sequence_id.unwrap(),
                        start: cur_start.unwrap(),
                        end: cur_end.unwrap(),
                        content: cur_content,
                        extra: cur_extra,
                    });

                    cur_sequence_id = None;
                    cur_start = None;
                    cur_end = None;
                    cur_extra = vec![];
                    cur_content = vec![];

                    state = State::ExpectSequenceId;
                    continue;
                }

                cur_content.push(line);
            },
        }
    }

    blocks.sort();
    blocks
}
