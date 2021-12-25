
pub struct Message {
    header: Header,
    body: Body,
}
struct Header{
    message_type: MessageType,
}

struct Body {
    content: [u8],
}

enum MessageType{
    AuthorityMessage,
    ControllerMessage,
    InterAgentMessage,
}

