// SPDX-License-Identifier: Apache-2.0

use std::{
  pin::Pin,
  sync::Arc,
  task::{Context, Poll},
};

use async_task::Runnable;
use tokio::{runtime::Handle, sync::mpsc::UnboundedSender};

pub(crate) type ForegroundTask = Box<dyn 'static + FnOnce()>;

#[derive(Debug)]
#[derive(Clone)]
pub(crate) struct ForegroundExecutor {
  tx: UnboundedSender<ForegroundTask>,
}
impl ForegroundExecutor {
  pub(crate) fn new(tx: UnboundedSender<ForegroundTask>) -> Self {
    Self { tx }
  }

  pub(crate) fn spawn<Fut, R>(&self, future: Fut) -> Task<R>
  where
    Fut: 'static + Future<Output = R>,
    R: 'static,
  {
    struct F<Fut> {
      future: Fut,
    }
    impl<Fut> Future for F<Fut>
    where
      Fut: Future,
    {
      type Output = Fut::Output;

      fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
      ) -> Poll<Self::Output> {
        unsafe { self.map_unchecked_mut(|this| &mut this.future).poll(cx) }
      }
    }

    let (runnable, task) = unsafe {
      async_task::Builder::new().spawn_unchecked(move |_| F { future }, {
        let tx = self.tx.clone();
        move |runnable: Runnable| {
          _ = tx.send(Box::new(move || {
            runnable.run();
          }));
        }
      })
    };
    runnable.schedule();
    Task(TaskState::Spawned(task))
  }
}

#[derive(Debug)]
#[derive(Clone)]
pub(crate) struct BackgroundExecutor {
  dispatcher: Option<Arc<Handle>>,
}
impl BackgroundExecutor {
  pub(crate) fn new() -> Self {
    Self { dispatcher: None }
  }
  pub(crate) fn pass_handle(&mut self, handle: Arc<Handle>) {
    self.dispatcher = Some(handle);
  }
  pub(crate) fn spawn<Fut>(&self, future: Fut) -> Task<Fut::Output>
  where
    Fut: 'static + Future + Send,
    Fut::Output: 'static + Send,
  {
    let (runnable, task) = async_task::Builder::new().spawn(move |_| future, {
      let dispatcher = self.dispatcher.clone().unwrap().clone();
      move |runnable: Runnable| {
        dispatcher.spawn(async move {
          runnable.run();
        });
      }
    });
    runnable.schedule();
    Task(TaskState::Spawned(task))
  }
}

#[derive(Debug)]
pub struct Task<T>(TaskState<T>);
impl<T> Task<T> {
  pub fn detach(self) {
    match self.0 {
      TaskState::Ready(..) => {}
      TaskState::Spawned(task) => task.detach(),
    };
  }
}
impl<T> Future for Task<T> {
  type Output = T;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    match unsafe {
      self
        .map_unchecked_mut(|task| &mut task.0)
        .get_unchecked_mut()
    } {
      TaskState::Ready(task) => Poll::Ready(task.take().unwrap()),
      TaskState::Spawned(task) => Pin::new(task).poll(cx),
    }
  }
}

#[derive(Debug)]
pub enum TaskState<T> {
  Ready(Option<T>),
  Spawned(async_task::Task<T>),
}
