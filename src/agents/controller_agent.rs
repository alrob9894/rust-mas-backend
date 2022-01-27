use std::thread;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;
use protobuf::Message;
use crate::messages::outs::controller_messages::{ControllerRound, ReqRegisterAgent, ReqRegisterAgent_SenderType, ReqRoundStatus, PubTerminationSignal};

pub struct ControllerAgent {
    context: zmq::Context,
    pub id: u32,
    iterations: u16,
}

impl ControllerAgent {
    pub fn new(
        id: u32,
        context: &zmq::Context,
        iterations: u16,
    ) -> ControllerAgent {
        ControllerAgent {
            context: context.clone(),
            id,
            iterations,
        }
    }

    pub fn run(&self, agent_number: u32) {
        let (tx_pub, rx_pub): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel();
        let (tx_rep, rx_rep): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel();
        let mut pub_worker = Some(self.publish_handler(rx_pub));
        let mut rep_worker = Some(self.reply_handler(tx_rep));
        wait_for_agents_to_register(&rx_rep, agent_number);
        for iter in 1..=self.iterations {
            println!("round {}", iter);
            self.send_round_to_agents(&tx_pub, iter);
            wait_for_agents_finish_round(&rx_rep, agent_number);
        }
        self.send_termination_signal(&tx_pub);
        drop(rep_worker.take());
        drop(pub_worker.take());
    }

    fn send_round_to_agents(&self, tx_pub: &Sender<Vec<u8>>, iteration: u16) {
        let mut msg = ControllerRound::new();
        msg.sender_id = self.id;
        msg.round = iteration as u32;
        tx_pub.send(msg.write_to_bytes().unwrap()).unwrap();
    }

    fn send_termination_signal(&self, tx_pub: &Sender<Vec<u8>>) {
        let mut msg = PubTerminationSignal::new();
        msg.sender_id = self.id;
        msg.msg = "Terminate".to_string();
        tx_pub.send(msg.write_to_bytes().unwrap()).unwrap();
    }

    pub fn publish_handler(&self, rx: Receiver<Vec<u8>>) {
        let context = self.context.clone();
        thread::spawn(move || {
            let publisher = context.socket(zmq::PUB).unwrap();
            assert!(publisher.bind("inproc://control_publisher").is_ok());
            'pubL: loop {
                match &rx.recv() {
                    Ok(msg) => { publisher.send(msg, 0).unwrap(); }
                    Err(err) => {
                        log::error!("{}", err);
                        break 'pubL;
                    }
                }
            }
        });
    }

    pub fn reply_handler(&self, tx: Sender<Vec<u8>>){
        let context = self.context.clone();
        thread::spawn(move || {
            let receiver = context.socket(zmq::REP).unwrap();
            assert!(receiver.bind("inproc://control_server").is_ok());
            'repL: loop {
                    match receiver.recv_bytes(0) {
                        Ok(msg) => {
                            tx.send(msg).unwrap();
                            receiver.send("", 0).unwrap();
                        }
                        Err(err) => {
                            log::error!("Reply handler{}", err);
                            break 'repL;
                        }
                    }
            }
        });
    }
}

fn wait_for_agents_to_register(rx: &Receiver<Vec<u8>>, agent_number: u32) -> bool {
    let mut agent_counter = 0;
    let mut stats_counter = 0;
    let mut authority_counter = 0;
    while agent_counter != agent_number || stats_counter != 1 || authority_counter != 1 {
        match rx.recv() {
            Ok(msg) => {
                let in_message = ReqRegisterAgent::parse_from_bytes(&msg).unwrap();
                if in_message.sender_type == ReqRegisterAgent_SenderType::SIMULATION_AGENT {
                    agent_counter += 1;
                } else if in_message.sender_type == ReqRegisterAgent_SenderType::STATISTIC_AGENT {
                    stats_counter += 1;
                } else if in_message.sender_type == ReqRegisterAgent_SenderType::AUTHORITY_AGENT {
                    authority_counter += 1;
                }
            }
            Err(err) => { log::error!("wait for reg{}", err) }
        }
    }
    true
}

fn wait_for_agents_finish_round(rx: &Receiver<Vec<u8>>, agent_number: u32) -> bool {
    let mut agent_counter = 0;
    while agent_counter < agent_number {
        match rx.recv() {
            Ok(msg) => {
                ReqRoundStatus::parse_from_bytes(&msg).unwrap();
                agent_counter += 1;
            }
            Err(err) => { log::error!("wait for finish{}", err) }
        }
    }
    true
}