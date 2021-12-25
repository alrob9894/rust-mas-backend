use crate::messages::message::Message;

trait AuthorityMessage{
    fn request_rules();
}

impl AuthorityMessage for Message {
    fn request_rules() {
        todo!()
    }
}