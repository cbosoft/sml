#[cfg(feature = "thread_safe")]
pub type RefCount<T> = std::sync::Arc<T>;

#[cfg(not(feature = "thread_safe"))]
pub type RefCount<T> = std::rc::Rc<T>;
