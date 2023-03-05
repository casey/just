use crate::RunResult;
use crossbeam::thread;

type ScopeResult<'src> = RunResult<'src, ()>;

pub(crate) struct TaskScope<'env, 'src, 'inner_scope> {
  inner: &'inner_scope thread::Scope<'env>,
  join_handles: Vec<thread::ScopedJoinHandle<'inner_scope, ScopeResult<'src>>>,
  parallel: bool,
}

impl<'env, 'src, 'inner_scope> TaskScope<'env, 'src, 'inner_scope> {
  pub(crate) fn run<'scope, F>(&'scope mut self, f: F) -> ScopeResult<'src>
  where
    'src: 'env,
    F: FnOnce() -> ScopeResult<'src>,
    F: Send + 'env,
  {
    if self.parallel {
      self.join_handles.push(self.inner.spawn(|_scope| f()));
      Ok(())
    } else {
      f()
    }
  }
}

/// task runner scope, based on `crossbeam::thread::scope`.
///
/// The `scope` object can be used to `.spawn` new tasks to be
/// run. The first error will be returned as result of this `task_scope`.
///
/// Only works for tasks with an `RunResult<'src, ()>` result type.
pub(crate) fn task_scope<'env, 'src, F>(parallel: bool, f: F) -> ScopeResult<'src>
where
  F: for<'inner_scope> FnOnce(&mut TaskScope<'env, 'src, 'inner_scope>) -> ScopeResult<'src>,
{
  thread::scope(|scope| {
    let mut task_scope = TaskScope {
      parallel,
      inner: scope,
      join_handles: Vec::new(),
    };

    f(&mut task_scope)?;

    for handle in task_scope.join_handles {
      handle.join().expect("could not join thread")?;
    }
    Ok(())
  })
  .expect("could not join thread")
}
