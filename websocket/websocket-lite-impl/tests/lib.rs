use std::sync::{Arc, Condvar, Mutex};

use condvar::ConditionalVariable;
use error::Error;
use traitcast_derive::Castable;
use websocket::{
    Closeable, ErrorCode, OnCloseSubscription, OnErrorSubscription, OnOpenSubscription,
    OnTextMessageSubscription, Openable, Subscriptions, TextSender,
};
use websocket_lite_impl::WebSocketLite;
///////////////////////////////////////////////////////////////////////////////
// Test open
#[derive(Castable)]
#[Traits(OnOpenSubscription)]
struct OnOpenTest {
    pub on_open_called: bool,
    pub condvar: Arc<(Mutex<bool>, Condvar)>,
}

impl OnOpenTest {
    fn default() -> OnOpenTest {
        OnOpenTest {
            on_open_called: false,
            condvar: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }
}
impl OnOpenSubscription for OnOpenTest {
    fn on_open(&mut self) {
        self.on_open_called = true;
        let (lock, cvar) = &*self.condvar;
        let mut started = lock.lock().unwrap();
        *started = true;
        cvar.notify_one();
    }
}

#[test]
fn invalid_url() {
    let mut ws = WebSocketLite::default();
    let result = ws.open("no url", None);
    assert!(result.is_err());
    assert_eq!(result.err(), Some(Error::new(ErrorCode::InvalidUrl, "")));
}
#[test]
fn unsupported_scheme() {
    let mut ws = WebSocketLite::default();
    let result = ws.open("http://www.google.de", None);
    assert!(result.is_err());
    assert_eq!(result.err(), Some(Error::new(ErrorCode::InvalidUrl, "")));
}
#[test]
fn supported_scheme() {
    let mut ws = WebSocketLite::default();
    let subscriber = Box::new(OnOpenTest::default());
    let pair = subscriber.condvar.clone();
    let subscriber1 = subscriber as Box<dyn traitcast::Castable>;
    let subscriber1 = Arc::new(Mutex::new(subscriber1));
    let clone = subscriber1.clone();

    assert!(ws.subscribe(clone).is_ok());
    let result = ws.open("ws://demo.piesocket.com/v3/channel_1?api_key=oCdCMcMPQpbvNjUIzqtvF1d2X2okWpDQj4AwARJuAgtjhzKxVEjQU6IdCjwm&notify_self", None);
    assert!(result.is_ok());
    let (lock, cvar) = &*pair;
    let mut started = lock.lock().unwrap();
    while !*started {
        started = cvar.wait(started).unwrap();
    }
    assert_eq!(
        subscriber1
            .lock()
            .unwrap()
            .as_ref()
            .query_ref::<OnOpenTest>()
            .unwrap()
            .on_open_called,
        true
    );
}
#[test]
fn supported_secured_scheme() {
    let mut ws = WebSocketLite::default();
    let subscriber = Box::new(OnOpenTest::default());
    let pair = subscriber.condvar.clone();
    let subscriber1 = subscriber as Box<dyn traitcast::Castable>;
    let subscriber1 = Arc::new(Mutex::new(subscriber1));
    let clone = subscriber1.clone();

    assert!(ws.subscribe(clone).is_ok());
    let result = ws.open("wss://demo.piesocket.com/v3/channel_1?api_key=oCdCMcMPQpbvNjUIzqtvF1d2X2okWpDQj4AwARJuAgtjhzKxVEjQU6IdCjwm&notify_self", None);
    assert!(result.is_ok());
    let (lock, cvar) = &*pair;
    let mut started = lock.lock().unwrap();
    while !*started {
        started = cvar.wait(started).unwrap();
    }
    assert_eq!(
        subscriber1
            .lock()
            .unwrap()
            .as_ref()
            .query_ref::<OnOpenTest>()
            .unwrap()
            .on_open_called,
        true
    );
}
///////////////////////////////////////////////////////////////////////////////
// Test interface error
#[derive(Castable)]
#[Traits(OnErrorSubscription)]
struct OnErrorTest {
    pub on_error_called: bool,
    pub condvar: Arc<(Mutex<bool>, Condvar)>,
}

impl OnErrorTest {
    fn default() -> OnErrorTest {
        OnErrorTest {
            on_error_called: false,
            condvar: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }
}
impl OnErrorSubscription for OnErrorTest {
    fn on_error(&mut self, _code: ErrorCode, _message: &str) {
        self.on_error_called = true;
        let (lock, cvar) = &*self.condvar;
        let mut started = lock.lock().unwrap();
        *started = true;
        cvar.notify_one();
    }
}
#[test]
fn could_not_open_url() {
    let mut ws = WebSocketLite::default();
    let subscriber = Box::new(OnErrorTest::default());
    let pair = subscriber.condvar.clone();
    let subscriber1 = subscriber as Box<dyn traitcast::Castable>;
    let subscriber1 = Arc::new(Mutex::new(subscriber1));
    let clone = subscriber1.clone();

    assert!(ws.subscribe(clone).is_ok());
    let result = ws.open("ws://abc.snoefried.com", None);
    assert!(result.is_ok());
    let (lock, cvar) = &*pair;
    let mut started = lock.lock().unwrap();
    while !*started {
        started = cvar.wait(started).unwrap();
    }
    assert_eq!(
        subscriber1
            .lock()
            .unwrap()
            .as_ref()
            .query_ref::<OnErrorTest>()
            .unwrap()
            .on_error_called,
        true
    );
}
///////////////////////////////////////////////////////////////////////////////
// Test interface close
#[derive(Castable)]
#[Traits(OnCloseSubscription, OnOpenSubscription)]
struct OnCloseTest {
    pub on_close_called: bool,
    pub condvar: Arc<(Mutex<bool>, Condvar)>,
}

impl OnCloseTest {
    fn default() -> OnCloseTest {
        OnCloseTest {
            on_close_called: false,
            condvar: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }
}
impl OnOpenSubscription for OnCloseTest {
    fn on_open(&mut self) {
        let (lock, cvar) = &*self.condvar;
        let mut started = lock.lock().unwrap();
        *started = true;
        cvar.notify_one();
    }
}
impl OnCloseSubscription for OnCloseTest {
    fn on_close(&mut self) {
        self.on_close_called = true;
        let (lock, cvar) = &*self.condvar;
        let mut started = lock.lock().unwrap();
        *started = true;
        cvar.notify_one();
    }
}
#[test]
fn connection_closed() {
    let mut ws = WebSocketLite::default();
    let subscriber = Box::new(OnCloseTest::default());
    let pair = subscriber.condvar.clone();
    let subscriber1 = subscriber as Box<dyn traitcast::Castable>;
    let subscriber1 = Arc::new(Mutex::new(subscriber1));
    let clone = subscriber1.clone();

    assert!(ws.subscribe(clone).is_ok());
    let result = ws.open("wss://demo.piesocket.com/v3/channel_1?api_key=oCdCMcMPQpbvNjUIzqtvF1d2X2okWpDQj4AwARJuAgtjhzKxVEjQU6IdCjwm&notify_self", None);
    assert!(result.is_ok());
    let (lock, cvar) = &*pair;
    {
        let mut started = lock.lock().unwrap();
        while !*started {
            started = cvar.wait(started).unwrap();
        }
        *started = false;
    }
    assert!(ws.close().is_ok());
    {
        let mut started = lock.lock().unwrap();
        while !*started {
            started = cvar.wait(started).unwrap();
        }
    }
    assert_eq!(
        subscriber1
            .lock()
            .unwrap()
            .as_ref()
            .query_ref::<OnCloseTest>()
            .unwrap()
            .on_close_called,
        true
    );
}
///////////////////////////////////////////////////////////////////////////////
// Test interface close
#[derive(Castable)]
#[Traits(OnTextMessageSubscription)]
struct OnMessageTest {
    pub on_message_called: bool,
    pub condvar: Arc<(Mutex<bool>, Condvar)>,
}

impl OnMessageTest {
    fn default() -> OnMessageTest {
        OnMessageTest {
            on_message_called: false,
            condvar: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }
}
impl OnTextMessageSubscription for OnMessageTest {
    fn on_message(&mut self, _message: &str) {
        self.on_message_called = true;
        let (lock, cvar) = &*self.condvar;
        let mut started = lock.lock().unwrap();
        *started = true;
        cvar.notify_one();
    }
}
#[test]
fn received_message() {
    let mut ws = WebSocketLite::default();
    let subscriber = Box::new(OnMessageTest::default());
    let pair = subscriber.condvar.clone();
    let subscriber1 = subscriber as Box<dyn traitcast::Castable>;
    let subscriber1 = Arc::new(Mutex::new(subscriber1));
    let clone = subscriber1.clone();

    assert!(ws.subscribe(clone).is_ok());
    let result = ws.open("wss://demo.piesocket.com/v3/channel_1?api_key=oCdCMcMPQpbvNjUIzqtvF1d2X2okWpDQj4AwARJuAgtjhzKxVEjQU6IdCjwm&notify_self", None);
    assert!(result.is_ok());
    let (lock, cvar) = &*pair;
    {
        let mut started = lock.lock().unwrap();
        while !*started {
            started = cvar.wait(started).unwrap();
        }
    }
    assert!(ws.close().is_ok());
    assert_eq!(
        subscriber1
            .lock()
            .unwrap()
            .as_ref()
            .query_ref::<OnMessageTest>()
            .unwrap()
            .on_message_called,
        true
    );
}
///////////////////////////////////////////////////////////////////////////////
// Test interface close
#[derive(Castable)]
#[Traits(OnTextMessageSubscription, OnOpenSubscription)]
struct OnMessageTest2 {
    pub on_message_called: bool,
    pub condvar: Arc<(Mutex<bool>, Condvar)>,
}

impl OnMessageTest2 {
    fn default() -> OnMessageTest2 {
        OnMessageTest2 {
            on_message_called: false,
            condvar: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }
}
impl OnOpenSubscription for OnMessageTest2 {
    fn on_open(&mut self) {
        let (lock, cvar) = &*self.condvar;
        let mut started = lock.lock().unwrap();
        *started = true;
        cvar.notify_one();
    }
}
impl OnTextMessageSubscription for OnMessageTest2 {
    fn on_message(&mut self, message: &str) {
        if !message.eq("Supidupi!") {
            return;
        }
        self.on_message_called = true;
        let (lock, cvar) = &*self.condvar;
        let mut started = lock.lock().unwrap();
        *started = true;
        cvar.notify_one();
    }
}
#[test]
fn received_sent_message() {
    let mut ws = WebSocketLite::default();
    let subscriber = Box::new(OnMessageTest2::default());
    let pair = subscriber.condvar.clone();
    let subscriber1 = subscriber as Box<dyn traitcast::Castable>;
    let subscriber1 = Arc::new(Mutex::new(subscriber1));
    let clone = subscriber1.clone();

    assert!(ws.subscribe(clone).is_ok());
    let result = ws.open("wss://demo.piesocket.com/v3/channel_1?api_key=oCdCMcMPQpbvNjUIzqtvF1d2X2okWpDQj4AwARJuAgtjhzKxVEjQU6IdCjwm&notify_self", None);
    assert!(result.is_ok());
    let (lock, cvar) = &*pair;
    {
        let mut started = lock.lock().unwrap();
        while !*started {
            started = cvar.wait(started).unwrap();
        }
        *started = false;
    }
    assert!(ws.send("Supidupi!").is_ok());
    {
        let mut started = lock.lock().unwrap();
        while !*started {
            started = cvar.wait(started).unwrap();
        }
    }
    assert_eq!(
        subscriber1
            .lock()
            .unwrap()
            .as_ref()
            .query_ref::<OnMessageTest2>()
            .unwrap()
            .on_message_called,
        true
    );
    assert!(ws.close().is_ok());
}
///////////////////////////////////////////////////////////////////////////////
// Test interface close
#[derive(Castable)]
#[Traits(OnOpenSubscription)]
struct OmniClient {
    pub on_open: ConditionalVariable<bool>,
}
impl OmniClient {
    fn default() -> OmniClient {
        OmniClient {
            on_open: ConditionalVariable::new(false),
        }
    }
}
impl OnOpenSubscription for OmniClient {
    fn on_open(&mut self) {
        self.on_open.notify(true);
    }
}
#[test]
fn connect_api() {
    let mut ws = WebSocketLite::default();
    let subscriber = Box::new(OmniClient::default());
    let on_open = subscriber.on_open.clone();
    let subscriber = subscriber as Box<dyn traitcast::Castable>;
    let subscriber = Arc::new(Mutex::new(subscriber));
    assert!(ws.subscribe(subscriber.clone()).is_ok());

    let result = ws.open("wss://nucleus.netallied.de/omni/api", None);
    assert!(result.is_ok());
    on_open.wait(true);
}
