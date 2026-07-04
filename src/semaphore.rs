use super::*;

#[derive(Clone)]
pub(crate) struct Semaphore(Arc<(Condvar, Mutex<u64>)>);

pub(crate) struct Guard(Semaphore);

impl Drop for Guard {
  fn drop(&mut self) {
    *self.0.mutex().lock().unwrap() += 1;
    self.0.condvar().notify_one();
  }
}

impl Semaphore {
  pub(crate) fn new(resource: NonZeroU64) -> Self {
    Self(Arc::new((Condvar::new(), Mutex::new(resource.into()))))
  }

  fn condvar(&self) -> &Condvar {
    &self.0.0
  }

  fn mutex(&self) -> &Mutex<u64> {
    &self.0.1
  }

  pub(crate) fn acquire(&self) -> Guard {
    let mut count = self
      .condvar()
      .wait_while(self.mutex().lock().unwrap(), |count| *count == 0)
      .unwrap();

    *count -= 1;

    Guard(self.clone())
  }
}
