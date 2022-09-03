use chrono::NaiveTime;

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub start: NaiveTime,
    pub end: NaiveTime,
    pub content: Vec<String>,
}
