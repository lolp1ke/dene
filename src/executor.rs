use std::{
  mem::ManuallyDrop,
  panic::Location,
  pin::Pin,
  sync::Arc,
  task::{Context, Poll},
  thread::{self, ThreadId},
};

use async_task::Runnable;
use tokio::{runtime::Handle, sync::mpsc::UnboundedSender};

pub(crate) type ForegroundTask = Box<dyn 'static + FnOnce()>;

#[derive(Debug)]
#[derive(Clone)]
pub struct ForegroundExecutor {
  tx: UnboundedSender<ForegroundTask>,
}
impl ForegroundExecutor {
  pub fn new(tx: UnboundedSender<ForegroundTask>) -> Self {
    Self { tx }
  }
  pub fn spawn<Fut, R>(&self, future: Fut) -> Task<R>
  where
    Fut: 'static + Future<Output = R>,
    R: 'static,
  {
    #[inline]
    fn thread_id() -> ThreadId {
      std::thread_local! {
          static ID: ThreadId = thread::current().id();
      }
      ID.try_with(|id| *id)
        .unwrap_or_else(|_| thread::current().id())
    }
    struct Checked<Fut> {
      thread_id: ThreadId,
      future: ManuallyDrop<Fut>,
      location: &'static Location<'static>,
    }
    impl<Fut> Future for Checked<Fut>
    where
      Fut: Future,
    {
      type Output = Fut::Output;

      fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
      ) -> Poll<Self::Output> {
        #[expect(unsafe_code, reason = "[`future`] is owned and never moved")]
        unsafe {
          self.map_unchecked_mut(|c| &mut *c.future).poll(cx)
        }
      }
    }
    impl<Fut> Drop for Checked<Fut> {
      fn drop(&mut self) {
        assert_eq!(
          self.thread_id,
          thread_id(),
          "task spawned at {}",
          self.location
        );

        #[expect(
          unsafe_code,
          reason = "[`Drop::drop`] runs once hence all good"
        )]
        unsafe {
          ManuallyDrop::drop(&mut self.future);
        };
      }
    }

    #[expect(
      unsafe_code,
      reason = "[`runnable`] is used and dropped on the original thread"
    )]
    let (runnable, task) = unsafe {
      async_task::Builder::new().spawn_unchecked(
        move |_| Checked {
          thread_id: thread_id(),
          future: ManuallyDrop::new(future),
          location: Location::caller(),
        },
        {
          let tx = self.tx.clone();
          move |runnable: Runnable| {
            _ = tx.send(Box::new(move || {
              runnable.run();
            }));
          }
        },
      )
    };
    runnable.schedule();
    Task(TaskState::Spawned(task))
  }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct BackgroundExecutor {
  dispatcher: Arc<Handle>,
}
impl BackgroundExecutor {
  pub fn new(dispatcher: Arc<Handle>) -> Self {
    Self { dispatcher }
  }

  pub fn spawn<Fut>(&self, future: Fut) -> Task<Fut::Output>
  where
    Fut: 'static + Future + Send,
    Fut::Output: 'static + Send,
  {
    let (runnable, task) = async_task::Builder::new().spawn(move |_| future, {
      let dispatcher = self.dispatcher.clone();
      move |runnable: Runnable| {
        dispatcher.spawn(async {
          runnable.run();
        });
      }
    });
    runnable.schedule();
    Task(TaskState::Spawned(task))
  }
}

#[derive(Debug)]
#[must_use = "[`Task`] do nothing unless detached using [`.detach()`] or awaited"]
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
    #[expect(
      unsafe_code,
      reason = "accessing inner [`task`] without moving it"
    )]
    match unsafe {
      self
        .map_unchecked_mut(|task| &mut task.0)
        .get_unchecked_mut()
    } {
      TaskState::Ready(ready) => Poll::Ready(ready.take().unwrap()),
      TaskState::Spawned(task) => Pin::new(task).poll(cx),
    }
  }
}

#[derive(Debug)]
pub enum TaskState<T> {
  Ready(Option<T>),
  Spawned(async_task::Task<T>),
}
