use std::sync::mpsc;
use std::sync::Arc;
use std::sync::mpsc::SendError;

pub fn prompt_queue(tx : &mpsc::Sender<Arc<String>>, prompt :Arc<String>) -> Result<(), SendError<Arc<String>>> {
    tx.send(prompt)
}
