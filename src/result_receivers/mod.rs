mod receiver_trait;
mod slack_sender;
mod terminal_sender;
mod qr;

pub use self::{
    receiver_trait::{
        ResultReceiver
    },
    slack_sender::{
        SlackResultSender
    },
    terminal_sender::{
        TerminalSender
    }
};