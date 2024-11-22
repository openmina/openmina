use std::any::Any;

pub trait InvariantService: redux::Service {
    type ClusterInvariantsState<'a>: 'a + std::ops::DerefMut<Target = InvariantsState>
    where
        Self: 'a;

    fn node_id(&self) -> usize {
        0
    }

    fn invariants_state(&mut self) -> &mut InvariantsState;

    fn cluster_invariants_state<'a>(&'a mut self) -> Option<Self::ClusterInvariantsState<'a>>
    where
        Self: 'a,
    {
        None
    }
}

#[derive(Default)]
pub struct InvariantsState(Vec<Box<dyn 'static + Send + Any>>);

impl InvariantsState {
    pub fn get<T: 'static + Send + Default>(&mut self, i: usize) -> &mut T {
        self.0.resize_with(i + 1, || Box::new(()));
        let v = self.0.get_mut(i).unwrap();
        if v.is::<T>() {
            v.downcast_mut().unwrap()
        } else {
            *v = Box::<T>::default();
            v.downcast_mut().unwrap()
        }
    }

    pub fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}
