use error::Error;
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::{debug, info};
use std::{
    collections::HashMap,
    hash::Hash,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Mutex, RwLock,
    },
};
use tokio::{runtime::Runtime, task::JoinHandle};
use url::Url;
use websocket::{
    BinarySender, Closeable, ErrorCode, OnBinaryMessageSubscription, OnCloseSubscription,
    OnErrorSubscription, OnOpenSubscription, OnTextMessageSubscription, Openable, Subscriptions,
    TextSender, ToBinary,
};
use websocket_lite::{AsyncClient, AsyncNetworkStream, ClientBuilder, Message, Opcode};

///////////////////////////////////////////////////////////////////////////////
/// This is the implementation of the websocket crate with webocket-lite
///
///////////////////////////////////////////////////////////////////////////////
/// Defining the states of the websocket
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionState {
    Closed = 0,
    Opening,
    Open,
    Closing,
}
///////////////////////////////////////////////////////////////////////////////
/// We need a struct that can be shared between the threads of the web socket
struct Connection {
    connection_state: AtomicU32,
    url: Option<Url>,
    header: Option<HashMap<String, String>>,
    send_receive_handle: Option<JoinHandle<Result<(), Error<ErrorCode>>>>,
    sender:
        Option<SplitSink<AsyncClient<Box<dyn AsyncNetworkStream + Sync + Send + Unpin>>, Message>>,
    subscribers: Vec<Arc<Mutex<Box<dyn traitcast::Castable>>>>,
}
///////////////////////////////////////////////////////////////////////////////
pub struct WebSocketLite {
    connection: Arc<RwLock<Connection>>,
    runtime: Runtime,
}
///////////////////////////////////////////////////////////////////////////////
/// At least a websocket is a state machine with four states.
/// Here the possible transitions are specified
impl WebSocketLite {
    pub fn default() -> WebSocketLite {
        let number_threads = 1;
        WebSocketLite {
            connection: Arc::new(RwLock::new(Connection {
                connection_state: AtomicU32::new(ConnectionState::Closed as u32),
                url: None,
                header: None,
                send_receive_handle: None,
                sender: None,
                subscribers: Vec::new(),
            })),
            runtime: tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(number_threads)
                .build()
                .unwrap(),
        }
    }
    ///////////////////////////////////////////////////////////////////////////////
    fn opening(&mut self) -> Result<(), Error<ErrorCode>> {
        let clone = self.connection.clone();
        let clone2 = self.connection.clone();
        let open_handle = self.runtime.spawn(Connection::connect(clone));

        let send_receive_handle = self
            .runtime
            .spawn(Connection::run_receive(clone2, open_handle));

        self.connection
            .write()
            .or_else(|err| {
                Err(Error::new(
                    ErrorCode::CouldNotSubscribe,
                    format!("Could not subscribe on_open ({err})").as_str(),
                ))
            })?
            .send_receive_handle = Some(send_receive_handle);

        Ok(())
    }
}
///////////////////////////////////////////////////////////////////////////////
/// Implementation of Subscriptions trait
impl Subscriptions for WebSocketLite {
    fn subscribe(
        &mut self,
        subscriber: Arc<Mutex<Box<dyn traitcast::Castable>>>,
    ) -> Result<(), Error<ErrorCode>> {
        self.connection
            .write()
            .or_else(|err| {
                Err(Error::new(
                    ErrorCode::CouldNotSubscribe,
                    format!("Could not subscribe on_open ({err})").as_str(),
                ))
            })?
            .subscribers
            .push(subscriber);
        Ok(())
    }
}
///////////////////////////////////////////////////////////////////////////////
/// Implementation of the Openable trait
impl Openable for WebSocketLite {
    fn open(
        &mut self,
        url_str: &str,
        header: Option<HashMap<String, String>>,
    ) -> Result<(), Error<ErrorCode>> {
        let url = Url::parse(url_str).or_else(|err| {
            Err(Error::<ErrorCode>::new(
                ErrorCode::InvalidUrl,
                format!("Could not parse url {}: {}", url_str, err).as_str(),
            ))
        })?;

        match url.scheme() {
            "ws" => (),
            "wss" => (),
            scheme => {
                return Err(Error::<ErrorCode>::new(
                    ErrorCode::InvalidUrl,
                    format!("Invalid scheme in url {}: {}", url_str, scheme).as_str(),
                ))
            }
        }

        {
            let mut connection = match self.connection.write() {
                Ok(guard) => guard,
                Err(err) => {
                    return Err(Error::new(
                        ErrorCode::CouldNotOpenConnection,
                        format!("Could not set url ({err})").as_str(),
                    ))
                }
            };
            connection.url = Some(url);
            connection.header = header;
        }

        self.opening()?;

        Ok(())
    }
}
///////////////////////////////////////////////////////////////////////////////
/// Implementation of Closeable
impl Closeable for WebSocketLite {
    fn close(&mut self) -> Result<(), Error<ErrorCode>> {
        {
            self.connection
                .write()
                .or_else(|err| {
                    Err(Error::new(
                        ErrorCode::CouldCloseConnection,
                        format!("Could not close ({err})").as_str(),
                    ))
                })?
                .connection_state
                .store(ConnectionState::Opening as u32, Ordering::Release);
        }
        let clone = self.connection.clone();
        self.runtime.spawn(Connection::close(clone));
        Ok(())
    }
}
///////////////////////////////////////////////////////////////////////////////
/// Implementation of TextSender
impl TextSender for WebSocketLite {
    fn send(&mut self, message: &str) -> Result<(), Error<ErrorCode>> {
        let clone = self.connection.clone();
        let msg = Message::new(Opcode::Text, message.to_string()).or_else(|err| {
            Err(Error::new(
                ErrorCode::CouldNotSendMessage,
                format!("Could not create text message ({err})").as_str(),
            ))
        })?;
        self.runtime.spawn(Connection::send(clone, msg));
        Ok(())
    }
}
///////////////////////////////////////////////////////////////////////////////
/// Implementation of BinarySender
impl BinarySender for WebSocketLite {
    fn send(&mut self, message: &dyn ToBinary) -> Result<(), Error<ErrorCode>> {
        let clone = self.connection.clone();
        let msg = Message::new(
            Opcode::Binary,
            message.to_bytes().ok_or_else(|| {
                Error::new(
                    ErrorCode::CouldNotSendMessage,
                    "Could not get bytes of message",
                )
            })?,
        )
        .or_else(|err| {
            Err(Error::new(
                ErrorCode::CouldNotSendMessage,
                format!("Could not create binray message ({err})").as_str(),
            ))
        })?;
        self.runtime.spawn(Connection::send(clone, msg));
        Ok(())
    }
}
///////////////////////////////////////////////////////////////////////////////
/// Implementation of Connection
impl Connection {
    async fn connect(
        conn: Arc<RwLock<Self>>,
    ) -> Result<AsyncClient<Box<dyn AsyncNetworkStream + Sync + Send + Unpin>>, Error<ErrorCode>>
    {
        let url = {
            let mut guard = conn.write().or_else(|err| {
                Err(Error::new(
                    ErrorCode::CouldNotOpenConnection,
                    format!("Could not connect to url ({err})").as_str(),
                ))
            })?;

            guard
                .connection_state
                .store(ConnectionState::Opening as u32, Ordering::Release);

            let url = guard.url.take().ok_or(Error::new(
                ErrorCode::CouldNotOpenConnection,
                "No url available",
            ))?;

            url
        };

        let builder = ClientBuilder::new(url.as_ref()).or_else(|err| {
            Err(Error::new(
                ErrorCode::CouldNotOpenConnection,
                format!("Could not open url {url}: {err}").as_str(),
            ))
        })?;

        let client = builder.async_connect().await.or_else(|err| {
            Err(Error::new(
                ErrorCode::CouldNotOpenConnection,
                format!("Could not create a client builder with url {url}: {err}").as_str(),
            ))
        })?;

        Ok(client)
    }
    ///////////////////////////////////////////////////////////////////////////////
    async fn run_receive(
        conn: Arc<RwLock<Self>>,
        open_handle: JoinHandle<
            Result<
                AsyncClient<Box<dyn AsyncNetworkStream + Sync + Send + Unpin>>,
                Error<ErrorCode>,
            >,
        >,
    ) -> Result<(), Error<ErrorCode>> {
        let client = open_handle
            .await
            .or_else(|err| {
                Err(Error::new(
                    ErrorCode::CouldNotOpenConnection,
                    format!("Could not open connection ({err})").as_str(),
                ))
            })?
            .or_else(|err| {
                Connection::notify_on_error(
                    &conn,
                    ErrorCode::CouldNotOpenConnection,
                    format!("Could not open connection ({:?})", err).as_str(),
                );
                Err(err)
            })?;

        let (sink, stream) = client.split::<Message>();

        {
            conn.write()
                .or_else(|err| {
                    Err(Error::new(
                        ErrorCode::CouldNotOpenConnection,
                        format!("Could not set sender ({err})").as_str(),
                    ))
                })?
                .sender = Some(sink);
        }

        Connection::notify_on_open(&conn);

        let result = futures::join!(Connection::receive(conn.clone(), stream));
        match result.0 {
            Ok(()) => (),
            Err(err) => Connection::notify_on_error(&conn, err.code, err.message.as_str()),
        }

        Connection::notify_on_close(&conn);
        Ok(())
    }
    ///////////////////////////////////////////////////////////////////////////////
    async fn send(conn: Arc<RwLock<Self>>, message: Message) -> Result<(), Error<ErrorCode>> {
        let mut sink = conn
            .write()
            .or_else(|err| {
                Err(Error::new(
                    ErrorCode::CouldNotSendMessage,
                    format!("Could not send message ({err})").as_str(),
                ))
            })?
            .sender
            .take()
            .ok_or_else(|| Error::new(ErrorCode::CouldNotSendMessage, "Sender not available"))?;

        sink.send(message).await.or_else(|err| {
            Err(Error::new(
                ErrorCode::CouldNotSendMessage,
                format!("Could not send message: ({err})").as_str(),
            ))
        })?;

        conn.write()
            .or_else(|err| {
                Err(Error::new(
                    ErrorCode::CouldNotSendMessage,
                    format!("Could not send message ({err})").as_str(),
                ))
            })?
            .sender = Some(sink);
        Ok(())
    }
    ///////////////////////////////////////////////////////////////////////////////
    async fn receive(
        conn: Arc<RwLock<Self>>,
        stream: SplitStream<AsyncClient<Box<dyn AsyncNetworkStream + Sync + Send + Unpin>>>,
    ) -> Result<(), Error<ErrorCode>> {
        let mut stream_mut = stream;
        loop {
            let (message, stream) = stream_mut.into_future().await;
            let msg = message
                .ok_or_else(|| {
                    Error::new(
                        ErrorCode::CouldNotReceiveMessage,
                        format!("Message is empty").as_str(),
                    )
                })?
                .or_else(|err| {
                    Err(Error::new(
                        ErrorCode::CouldNotReceiveMessage,
                        format!("Error while receiving ({err})").as_str(),
                    ))
                })?;

            match msg.opcode() {
                Opcode::Text => {
                    debug!(target: "websocket-lite-impl", "Received text message");
                    let text = msg.as_text().ok_or_else(|| {
                        Error::new(ErrorCode::CouldNotReceiveMessage, "Text message is no text")
                    })?;
                    Connection::notify_on_text_message(&conn, text);
                }
                Opcode::Binary => {
                    debug!(target: "websocket-lite-impl", "Received binary message");
                    let data = msg.data();
                    Connection::notify_on_binary_message(&conn, data);
                }
                Opcode::Ping => {
                    debug!(target: "websocket-lite-impl", "Received ping message");
                    Connection::send(conn.clone(), Message::pong(msg.into_data())).await?;
                }
                Opcode::Close => {
                    info!(target: "websocket-lite-impl", "Received close message");
                    break;
                }
                _ => (),
            }
            stream_mut = stream;
        }
        Ok(())
    }
    ///////////////////////////////////////////////////////////////////////////////
    async fn close(conn: Arc<RwLock<Self>>) -> Result<(), Error<ErrorCode>> {
        let handle = {
            let mut connection = conn.write().or_else(|err| {
                Err(Error::new(
                    ErrorCode::CouldCloseConnection,
                    format!("Could not get close handle ({err})").as_str(),
                ))
            })?;

            let handle = connection.send_receive_handle.take().ok_or_else(|| {
                Error::new(
                    ErrorCode::CouldCloseConnection,
                    "Close handle not available",
                )
            })?;
            handle
        };

        Connection::send(conn.clone(), Message::close(None)).await?;

        handle.await.or_else(|err| {
            Err(Error::new(
                ErrorCode::CouldCloseConnection,
                format!("Close failed ({err})").as_str(),
            ))
        })??;

        Ok(())
    }
    ///////////////////////////////////////////////////////////////////////////////
    fn notify_on_open(conn: &Arc<RwLock<Self>>) {
        match conn.read() {
            Ok(guard) => {
                guard
                    .connection_state
                    .store(ConnectionState::Open as u32, Ordering::Release);
                guard.subscribers.iter().for_each(|subscriber| {
                    match subscriber.lock() {
                        Ok(mut guard) =>{
                            match guard.query_mut::<dyn OnOpenSubscription>() {
                                Some(open) => open.on_open(),
                                None => ()
                            }
                        }
                        Err(err) => log::error!(target: "websocket-lite-impl", "Could not notify open for a subscriber ({err})")
                    }
                })
            }
            Err(err) => {
                log::error!(target: "websocket-lite-impl", "Could not notify open for a subscriber ({err})")
            }
        }
    }
    ///////////////////////////////////////////////////////////////////////////////
    fn notify_on_error(conn: &Arc<RwLock<Self>>, code: ErrorCode, message: &str) {
        match conn.read() {
            Ok(guard) => {
                guard.subscribers.iter().for_each(|subscriber| {
                    match subscriber.lock() {
                        Ok(mut guard) =>{
                            match guard.query_mut::<dyn OnErrorSubscription>() {
                                Some(open) => open.on_error(code, message),
                                None => ()
                            }
                        }
                        Err(err) => log::error!(target: "websocket-lite-impl", "Could not error open for a subscriber ({err})")
                    }
                });
            }
            Err(err) => {
                log::error!(target: "websocket-lite-impl", "Could not notify error for a subscriber ({err})")
            }
        }
    }
    ///////////////////////////////////////////////////////////////////////////////
    fn notify_on_close(conn: &Arc<RwLock<Self>>) {
        match conn.read() {
            Ok(guard) => {
                guard
                    .connection_state
                    .store(ConnectionState::Closed as u32, Ordering::Release);
                guard.subscribers.iter().for_each(|subscriber| {
                    match subscriber.lock() {
                        Ok(mut guard) =>{
                            match guard.query_mut::<dyn OnCloseSubscription>() {
                                Some(open) => open.on_close(),
                                None => ()
                            }
                        }
                        Err(err) => log::error!(target: "websocket-lite-impl", "Could not notify close for a subscriber ({err})")
                    }
                })
            }
            Err(err) => {
                log::error!(target: "websocket-lite-impl", "Could not notify close for a subscriber ({err})")
            }
        }
    }
    ///////////////////////////////////////////////////////////////////////////////
    fn notify_on_text_message(conn: &Arc<RwLock<Self>>, message: &str) {
        match conn.read() {
            Ok(guard) => {
                guard
                    .connection_state
                    .store(ConnectionState::Closed as u32, Ordering::Release);
                guard.subscribers.iter().for_each(|subscriber| {
                match subscriber.lock() {
                    Ok(mut guard) =>{
                        match guard.query_mut::<dyn OnTextMessageSubscription>() {
                            Some(open) => open.on_message(message),
                            None => ()
                        }
                    }
                    Err(err) => log::error!(target: "websocket-lite-impl", "Could not notify close for a subscriber ({err})")
                }
            })
            }
            Err(err) => {
                log::error!(target: "websocket-lite-impl", "Could not notify close for a subscriber ({err})")
            }
        }
    }
    ///////////////////////////////////////////////////////////////////////////////
    fn notify_on_binary_message(conn: &Arc<RwLock<Self>>, message: &[u8]) {
        match conn.read() {
            Ok(guard) => {
                guard
                    .connection_state
                    .store(ConnectionState::Closed as u32, Ordering::Release);
                guard.subscribers.iter().for_each(|subscriber| {
                match subscriber.lock() {
                    Ok(mut guard) =>{
                        match guard.query_mut::<dyn OnBinaryMessageSubscription>() {
                            Some(open) => open.on_message(message),
                            None => ()
                        }
                    }
                    Err(err) => log::error!(target: "websocket-lite-impl", "Could not notify close for a subscriber ({err})")
                }
            })
            }
            Err(err) => {
                log::error!(target: "websocket-lite-impl", "Could not notify close for a subscriber ({err})")
            }
        }
    }
}
