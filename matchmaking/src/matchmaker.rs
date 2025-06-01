use std::sync::mpsc;
use std::thread;
use std::sync::Arc;

pub fn init_matchmaker()-> mpsc::Sender<Arc<String>> {
    let (tx, rx) = mpsc::channel::<Arc<String>>();
    use std::thread;
    thread::spawn(move || {
        make_match(rx);
    });
    tx
}


fn make_match(rx: mpsc::Receiver<Arc<String>>) {
    let mut queue = Vec::new();
    loop{
        match rx.try_recv(){
            Ok(msg) => queue.push(msg),
            Err(_e) => (),
        }
        if queue.len() >= 2 {
            let person1 = queue.remove(0);
            let person2 = queue.remove(0);
            println!("Matched {} with {}", person1, person2);
        }
        thread::sleep(std::time::Duration::from_millis(10));
    }
}
