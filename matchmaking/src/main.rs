use std::sync::Arc;

pub mod matchmaker;
pub mod promptqueue;

pub fn main() {
    let ts = matchmaker::init_matchmaker();
    let mut n = 0;
    loop{
        let _ = promptqueue::prompt_queue(&ts, Arc::new(String::from("person") + &n.to_string()));
        n += 1;
    }
}
