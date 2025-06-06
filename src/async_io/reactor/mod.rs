use std::collections::HashMap;
use std::fmt::Debug;
use std::os::windows::prelude::AsRawSocket;
use std::sync::{Arc, RwLock};
use std::task::{Context, Waker};
use std::time::Duration;
use polling::{AsRawSource, AsSource, Event, Events, Poller};
use anyhow::Result;
use once_cell::sync::Lazy;

pub static REACTOR: Lazy<Arc<RwLock<Reactor>>> = Lazy::new(|| {
    Arc::new(RwLock::new(Reactor::new()))
});


pub struct Reactor {
    readable: HashMap<usize, Vec<Waker>>,
    writable: HashMap<usize, Vec<Waker>>,
    poller: Poller
}

impl Default for Reactor {
    fn default() -> Self {
        Self::new()
    }
}

impl Reactor {
    pub fn new() -> Reactor {
        Reactor {
            readable: Default::default(),
            writable: Default::default(),
            poller: Poller::new().unwrap(),
        }
    }

    pub fn get_interest(&self, key: &impl AsRawSource) -> Event {
        let key = key.raw() as usize;
        let is_readable = self.readable.contains_key(&key);
        let is_writable = self.writable.contains_key(&key);

        match (is_readable, is_writable) {
            (false, false) => Event::none(key),
            (false, true) => Event::writable(key),
            (true, false) => Event::readable(key),
            (true, true) => Event::all(key),
        }
    }

    pub fn add(&self, key: impl AsRawSource) {
        let event = self.get_interest(&key);
        unsafe { self.poller.add(key, event).unwrap() }
    }

    pub fn remove(&mut self, source: impl AsSource) {
        let key = source.source().as_raw_socket() as usize;
        self.poller.delete(source).unwrap();
        self.readable.remove(&key);
        self.writable.remove(&key);
    }

    pub fn wake_on_readable(&mut self, source: impl AsSource + Debug, ctx: &mut Context) {
        let key = source.source().as_raw_socket();
        self.readable
            .entry(key as usize)
            .or_default()
            .push(ctx.waker().clone());

        println!("{source:?}");
        
        self.poller.modify(source, self.get_interest(&key)).unwrap();
    }

    pub fn wake_on_writable(&mut self, source: impl AsSource, ctx: &mut Context) {
        let key = source.source().as_raw_socket();
        self.writable
            .entry(key as usize)
            .or_default()
            .push(ctx.waker().clone());

        self.poller.modify(source, self.get_interest(&key)).unwrap()
    }

    pub fn drain_wakers(&mut self, events: Events) -> Vec<Waker> {
        let mut wakers = vec![];

        for ev in events.iter() {
            if let Some((_, readers)) = self.readable.remove_entry(&ev.key) {
                for reader in readers {
                    wakers.push(reader);
                }
            }

            if let Some((_, writers)) = self.writable.remove_entry(&ev.key) {
                for writer in writers {
                    wakers.push(writer);
                }
            }
        }

        wakers
    }

    pub fn wait(&self, events: &mut Events, timeout: Option<Duration>) -> Result<usize> {
        Ok(self.poller.wait(events, timeout)?)
    }
    
    pub fn waiting_on_events(&self) -> bool {
        !self.readable.is_empty() || !self.writable.is_empty()
    }
}