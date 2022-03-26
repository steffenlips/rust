use error::Error;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

///////////////////////////////////////////////////////////////////////////////
/// WebSocket client
/// This crate defines the trait, callbacks and error codes
/// The implementation are defined in sub crates depending on the raw websocket
/// implementation to use. Therefore, implementations may be updated or changed
/// in the future.
///
///
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ErrorCode {
    InvalidUrl,
    CouldNotOpenConnection,
    CouldNotSubscribe,
    CouldNotNotifySubscribers,
    CouldNotStartCommunication,
    CouldNotReceiveMessage,
    CouldNotSendMessage,
    CouldCloseConnection,
}

///////////////////////////////////////////////////////////////////////////////
/// A struct that uses a web socket should implement these traits to get
/// notified when somethings happen on the connection, but it can decide which
/// subscription are of interest-.
///
/// Subscription that notifies a connection is open. (This callback is also
/// called, when a connection is already open, but the subscription happens later.)
pub trait OnOpenSubscription: traitcast::Castable {
    fn on_open(&mut self);
}

///////////////////////////////////////////////////////////////////////////////
/// Subscription that notifies a connection is closed. (This callback is also
/// called, when a connection is already closed, but the subscription happens
/// later).
pub trait OnCloseSubscription {
    fn on_close(&mut self);
}

///////////////////////////////////////////////////////////////////////////////
/// Subscription that notifies any error on the connection, like could not
/// connect, terminated abnormally, ...
pub trait OnErrorSubscription {
    fn on_error(&mut self, code: ErrorCode, message: &str);
}

///////////////////////////////////////////////////////////////////////////////
/// Subscription that notifies a text message
pub trait OnTextMessageSubscription {
    fn on_message(&mut self, message: &str);
}

///////////////////////////////////////////////////////////////////////////////
/// Subscription that notifies a binary message
pub trait OnBinaryMessageSubscription {
    fn on_message(&mut self, message: &[u8]);
}

///////////////////////////////////////////////////////////////////////////////
/// A WebSocket implementation should implement the following traits.
/// This trait handles the suscription stuff
pub trait Subscriptions {
    fn subscribe(
        &mut self,
        subscriber: Arc<Mutex<Box<dyn traitcast::Castable>>>,
    ) -> Result<(), Error<ErrorCode>>;
}

///////////////////////////////////////////////////////////////////////////////
/// The web socket should implement this trait only, when it is in an
/// unconnected state.
pub trait Openable {
    fn open(
        &mut self,
        url: &str,
        header: Option<HashMap<String, String>>,
    ) -> Result<(), Error<ErrorCode>>;
}

///////////////////////////////////////////////////////////////////////////////
/// The web socket should implement this trait only, when it is in a connected
/// state
pub trait Closeable {
    fn close(&mut self) -> Result<(), Error<ErrorCode>>;
}

///////////////////////////////////////////////////////////////////////////////
/// The web socket can implement this trait if it supports text message sending
pub trait TextSender {
    fn send(&mut self, message: &str) -> Result<(), Error<ErrorCode>>;
}

///////////////////////////////////////////////////////////////////////////////
/// The web socket can implement this trait if it supports binary message
/// sending
pub trait BinarySender {
    fn send(&mut self, message: &dyn ToBinary) -> Result<(), Error<ErrorCode>>;
}

///////////////////////////////////////////////////////////////////////////////
/// Anything that has a binary representation should implement this trait
/// if it should be send
pub trait ToBinary {
    fn to_bytes(&self) -> Option<Vec<u8>>;
}
