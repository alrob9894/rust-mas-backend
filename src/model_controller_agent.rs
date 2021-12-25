use std::thread;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;
use crate::AGENT_NUMBER;

// #[derive(Copy, Clone)]
pub struct ControllerAgent {
    pub id: u32,
    iterations: u16,
}

impl ControllerAgent {
    pub fn new(
        id: u32,
        iterations: u16,
    ) -> ControllerAgent { ControllerAgent {
        id,
        iterations,
    }}

    pub fn run_sim(&self) {
        // let context = zmq::Context::new();
        // let xmitter = context.socket(zmq::PAIR).unwrap();
        // xmitter
        //     .connect("inproc://agent")
        //     .expect("controller failed connecting");
        // println!("Controller ready, signalling agent");
        // xmitter.send("Agent is ready", 0).expect("failed sending");



        let (tx_pub, rx_pub): (Sender<String>, Receiver<String>) = mpsc::channel();
        let (tx_rep, rx_rep): (Sender<String>, Receiver<String>) = mpsc::channel();
        self.publish_handler(rx_pub);
        self.reply_handler(tx_rep);
        // todo: hier irgendwie die IDs oder ähnliches von den Agenten sammeln
        self.wait_for_agents(&rx_rep);
        for iter in 1..=self.iterations {  //self.iterations
            println!("Iteration: {}", iter);
            tx_pub.send(iter.to_string()).unwrap();
            self.wait_for_agents(&rx_rep);
            // let string = rx_rep.recv().unwrap();
            // println!("{} in round: {}", string, iter);
        }
    }

    fn wait_for_agents(&self, rx: &Receiver<String>) -> bool {
        let mut agent_counter = 0;
        while agent_counter < AGENT_NUMBER {
            match rx.recv() {
                Ok(msg) => {
                    agent_counter += 1;}
                Err(err) => {}
            }
        }
        true
    }

    pub fn publish_handler(&self, rx: Receiver<String>) {
        {
            let handle = thread::spawn(move|| {
                let context = zmq::Context::new();
                let publisher = context.socket(zmq::PUB).unwrap();
                assert!(publisher.bind("tcp://*:5555").is_ok());
                loop {
                    match &rx.recv() {
                        Ok(msg) => {publisher.send(msg,0).unwrap();}
                        Err(error) => {}
                    }
                }
            });
        }
    }

    pub fn reply_handler(&self, tx: Sender<String>) {
        {
            let handle = thread::spawn(move || {
                let context = zmq::Context::new();
                let receiver = context.socket(zmq::REP).unwrap();
                assert!(receiver.bind("tcp://*:5556").is_ok());
                loop {
                    match receiver.recv_string(0) {
                        Ok(msg) => {
                            tx.send(msg.unwrap());
                            receiver.send("",0);}
                        Err(err) => {}
                    }
                }
            });
        }
    }
}