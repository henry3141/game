pub struct Channel<T> where T: std::marker::Send {
    pub sender: std::sync::mpsc::Sender<T>,
    pub receiver: std::sync::mpsc::Receiver<T>,
}

impl<T> Channel<T> where T: std::marker::Send {
    pub fn new() -> (Self,Self) {
        let (s1,r1) = std::sync::mpsc::channel();
        let (s2,r2) = std::sync::mpsc::channel();
        (Self { sender: s1, receiver: r2 }, Self { sender: s2, receiver: r1 })
    }

    pub fn send(&self, data:T) {
        self.sender.send(data).unwrap();
    }

    pub fn recv(&self) -> T {
        self.receiver.recv().unwrap()
    }

    pub fn try_recv(&self) -> Option<T> {
        self.receiver.try_recv().ok()
    }
}