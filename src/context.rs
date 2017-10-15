extern crate std;

extern crate futures;
extern crate futures_spawn;

use self::futures::*;
use self::futures_spawn::*;

use std::sync::{Arc, RwLock};

pub type ContextData<V> = Arc<RwLock<Option<V>>>;

pub type ContextFuture<T> = Box<Future<Item = T, Error = ()>>;

pub struct ContextMonad<V, T: Send + 'static> {
    pub f: Box<Fn() -> (ContextData<V>, T) + Send + 'static>,
}

impl<V: 'static, T: Send + 'static> ContextMonad<V, T> {
    pub fn bind<F, U>(self, func: F) -> ContextMonad<V, U>
    where
        F: Fn(Option<&V>, T) -> U,
        F: Send + 'static,
        U: Send,
    {
        ContextMonad::<V, U> {
            f: Box::new(move || {
                let (c, t) = (self.f)();
                let u = func(c.read().unwrap().as_ref(), t);
                (c, u)
            }),
        }
    }

    pub fn bind_mut<F, U>(self, func: F) -> ContextMonad<V, U>
    where
        F: Fn(Option<&mut V>, T) -> U,
        F: Send + 'static,
        U: Send,
    {
        ContextMonad::<V, U> {
            f: Box::new(move || {
                let (c, t) = (self.f)();
                let u = func(c.write().unwrap().as_mut(), t);
                (c, u)
            }),
        }
    }

    pub fn bind_rwlock<F, U>(self, func: F) -> ContextMonad<V, U>
    where
        F: Fn(&RwLock<Option<V>>, T) -> U,
        F: Send + 'static,
        U: Send,
    {
        ContextMonad::<V, U> {
            f: Box::new(move || {
                let (c, t) = (self.f)();
                let u = func(&c, t);
                (c, u)
            }),
        }
    }

    pub fn assemble(self) -> Box<Fn() -> T + Send + 'static> {
        Box::new(move || (self.f)().1)
    }

    pub fn run(self) -> ContextFuture<T> {
        Box::from(NewThread.spawn(futures::lazy({
            move || Ok(self.assemble()())
        })))
    }
}

pub struct Context<V: Send + Sync + 'static> {
    data: ContextData<V>,
}

impl<V: Send + Sync + 'static> Drop for Context<V> {
    fn drop(&mut self) {
        *self.data.write().unwrap() = None;
    }
}

impl<V: Send + Sync + Default> Default for Context<V> {
    fn default() -> Self {
        Context::new(Default::default())
    }
}

impl<V: Send + Sync + 'static> Context<V> {
    pub fn new(v: V) -> Self {
        Self { data: Arc::new(RwLock::new(Some(v))) }
    }

    pub fn raw(&self) -> &RwLock<Option<V>> {
        &*self.data
    }

    pub fn compose(&self) -> ContextMonad<V, ()> {
        ContextMonad {
            f: {
                Box::new({
                    let data = self.data.clone();
                    move || (data.clone(), ())
                })
            },
        }
    }

    pub fn run<F, T>(&self, f: F) -> ContextFuture<T>
    where
        F: Fn(Option<&V>) -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        self.compose().bind(move |data, _| f(data)).run()
    }

    pub fn run_force<F, T>(&self, f: F) -> ContextFuture<T>
    where
        F: Fn(&V) -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        self.compose().bind(move |data, _| f(data.unwrap())).run()
    }

    pub fn run_mut<F, T>(&self, f: F) -> ContextFuture<T>
    where
        F: Fn(Option<&mut V>) -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        self.compose().bind_mut(move |data, _| f(data)).run()
    }

    pub fn run_mut_force<F, T>(&self, f: F) -> ContextFuture<T>
    where
        F: Fn(&mut V) -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        self.compose()
            .bind_mut(move |data, _| f(data.unwrap()))
            .run()
    }
}
