use error::Error;
use log::error;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use traitcast::Castable;

use crate::service::{Service, ServiceFactory};

#[derive(PartialEq, Debug)]
pub enum ErrorCode {
    Uninitialized,
    UnregisteredService,
    AlreadyRegisteredService,
    CouldNotCreateSessionService,
    CouldNotGetSessionService,
    CouldNotUninitialize,
    Unimplemented,
    ServiceError,
}

pub trait Session {
    fn key(&self) -> u32;
}
pub struct SimpleSession {
    key: u32,
}
impl SimpleSession {
    pub const fn default() -> SimpleSession {
        SimpleSession { key: 0 }
    }

    pub fn new() -> SimpleSession {
        SimpleSession {
            key: SESSION_COUNTER.fetch_add(1, Ordering::Relaxed),
        }
    }
}
impl Session for SimpleSession {
    fn key(&self) -> u32 {
        self.key
    }
}

static SESSION_COUNTER: AtomicU32 = AtomicU32::new(1);

pub struct Registry {
    registered_service_factories: HashMap<String, ServiceFactory>,
    available_sessions: HashMap<u32, HashMap<String, Arc<Mutex<Box<dyn Castable>>>>>,
}

impl Registry {
    pub fn default() -> Registry {
        Registry {
            registered_service_factories: HashMap::new(),
            available_sessions: HashMap::new(),
        }
    }

    pub fn register_service<Impl: Service + ?Sized>(
        prototype: ServiceFactory,
    ) -> Result<(), Error<ErrorCode>> {
        let registry = &mut REGISTRY_INSTANCE.lock().or_else(|err| {
            Err(Error::<ErrorCode>::new(
                ErrorCode::Uninitialized,
                format!("Di container not initialized: {}", err).as_str(),
            ))
        })?;

        let name = std::any::type_name::<Impl>().to_owned();

        if registry.registered_service_factories.contains_key(&name) {
            return Err(Error::new(
                ErrorCode::AlreadyRegisteredService,
                format!("Service {} already exists", name).as_str(),
            ));
        }

        registry
            .registered_service_factories
            .insert(name, prototype);

        Ok(())
    }

    pub fn unregister_service<Impl: Service + ?Sized>() -> Result<(), Error<ErrorCode>> {
        let registry = &mut REGISTRY_INSTANCE.lock().or_else(|err| {
            Err(Error::<ErrorCode>::new(
                ErrorCode::Uninitialized,
                format!("Di container not initialized: {}", err).as_str(),
            ))
        })?;

        let name = std::any::type_name::<Impl>().to_owned();

        registry.registered_service_factories.remove(&name);

        registry.available_sessions.iter_mut().for_each(|map| {
            let service = map.1.remove(&name);
            if service.is_some() {
                Registry::unitialize_service(&service.unwrap(), name.as_str());
            }
        });

        Ok(())
    }

    pub fn clear_session(session: &dyn Session) -> Result<(), Error<ErrorCode>> {
        let registry = &mut REGISTRY_INSTANCE.lock().or_else(|err| {
            Err(Error::<ErrorCode>::new(
                ErrorCode::Uninitialized,
                format!("Di container not initialized: {}", err).as_str(),
            ))
        })?;

        let session = registry.available_sessions.remove(&session.key());
        if session.is_some() {
            registry.uninitialize_services_of_session(session.unwrap());
        }
        Ok(())
    }

    pub fn get_service<Impl: 'static + Service + ?Sized>(
        session: &dyn Session,
    ) -> Result<Arc<Mutex<Box<dyn Castable>>>, Error<ErrorCode>> {
        let registry = &mut REGISTRY_INSTANCE.lock().or_else(|err| {
            Err(Error::<ErrorCode>::new(
                ErrorCode::Uninitialized,
                format!("Di container not initialized: {}", err).as_str(),
            ))
        })?;

        if !registry.available_sessions.contains_key(&session.key()) {
            registry
                .available_sessions
                .insert(session.key(), HashMap::new());
        }

        let name = std::any::type_name::<Impl>().to_string();

        let service_instance = registry.get_session_service::<Impl>(session, &name)?;

        Ok(service_instance.clone())
    }
    fn get_session_service<Impl: 'static + Service + ?Sized>(
        &mut self,
        session: &dyn Session,
        service_name: &String,
    ) -> Result<&Arc<Mutex<Box<dyn Castable>>>, Error<ErrorCode>> {
        let session_services =
            self.available_sessions
                .get_mut(&session.key())
                .ok_or_else(|| {
                    Error::<ErrorCode>::new(
                        ErrorCode::CouldNotCreateSessionService,
                        format!(
                            "Could not create session service for session {}",
                            session.key()
                        )
                        .as_str(),
                    )
                })?;

        if !session_services.contains_key(service_name) {
            let factory = self
                .registered_service_factories
                .get(service_name)
                .ok_or_else(|| {
                    Error::new(
                        ErrorCode::UnregisteredService,
                        format!("Unregistered service {}", service_name).as_str(),
                    )
                })?;

            let mut service_instance = factory();
            match service_instance.as_mut().query_mut::<Impl>() {
                Some(service) => service.initialize(),
                None => (),
            }

            session_services.insert(
                service_name.to_string(),
                Arc::new(Mutex::new(service_instance)),
            );
        }

        let service_instance = session_services.get(service_name).ok_or_else(|| {
            Error::<ErrorCode>::new(
                ErrorCode::CouldNotCreateSessionService,
                format!(
                    "Could not create session service for session {}",
                    session.key()
                )
                .as_str(),
            )
        })?;

        Ok(service_instance)
    }

    fn uninitialize_services_of_session(
        &self,
        session: HashMap<String, Arc<Mutex<Box<dyn Castable>>>>,
    ) {
        session
            .iter()
            .for_each(|service| Registry::unitialize_service(service.1, service.0));
    }

    fn unitialize_service(service: &Arc<Mutex<Box<dyn Castable>>>, name: &str) {
        match service.lock() {
            Ok(mut guard) => match guard.query_mut::<dyn Service>() {
                Some(service) => service.uninitialize(),
                None => error!(target: "di", "Could not uninitialize service {}", name),
            },
            Err(err) => {
                error!(target: "di", "Could not uninitialize service {} ({})", name, err)
            }
        }
    }
}

/// The one and only singleton of the DI registration
static REGISTRY_INSTANCE: Lazy<Arc<Mutex<Registry>>> =
    Lazy::new(|| Arc::new(Mutex::new(Registry::default())));
