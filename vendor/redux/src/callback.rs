#[cfg(feature = "serializable_callbacks")]
use linkme::distributed_slice;

pub use paste;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

pub struct AnyAction(pub Box<dyn std::any::Any>);

#[cfg(feature = "serializable_callbacks")]
#[distributed_slice]
pub static CALLBACKS: [(&str, fn(&str, Box<dyn std::any::Any>) -> AnyAction)];

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct Callback<T> {
    #[serde(skip, default = "default_fun_ptr")]
    fun_ptr: Option<fn(T) -> AnyAction>,
    pub fun_name: Cow<'static, str>,
}

fn default_fun_ptr<T>() -> Option<fn(T) -> AnyAction> {
    None
}

impl<T: 'static> Callback<T> {
    pub fn new(name: &'static str, fun_ptr: fn(T) -> AnyAction) -> Self {
        Self {
            fun_ptr: Some(fun_ptr),
            fun_name: Cow::Borrowed(name),
        }
    }

    pub fn call<Action>(&self, args: T) -> Action
    where
        Action: From<AnyAction>,
    {
        if let Some(fun) = self.fun_ptr {
            return fun(args).into();
        }

        #[cfg(not(feature = "serializable_callbacks"))]
        unimplemented!();

        #[cfg(feature = "serializable_callbacks")]
        {
            // We reach this point only when the callback was deserialized
            for (name, fun) in CALLBACKS {
                if name == &self.fun_name {
                    return fun(std::any::type_name::<T>(), Box::new(args)).into();
                }
            }

            panic!("callback function {} not found", self.fun_name)
        }
    }
}

#[macro_export]
macro_rules! _callback {
    ($callback_name:ident, $action_ty:ty, $arg:tt, $arg_type:ty, $body:expr) => {{
        use $crate::{AnyAction, Callback};

        #[cfg(feature = "serializable_callbacks")]
        use {$crate::CALLBACKS, linkme::distributed_slice};

        redux::paste::paste! {
            #[allow(unused)] // $arg is marked as unused, but it's used in `$body`
            fn convert_impl($arg: $arg_type) -> AnyAction {
                let action: $action_ty = ($body).into();
                AnyAction(Box::new(action))
            }

            fn $callback_name(call_type: &str, args: Box<dyn std::any::Any>) -> AnyAction {
                #[cfg(feature = "serializable_callbacks")]
                {
                    #[distributed_slice(CALLBACKS)]
                    static CALLBACK_DESERIALIZE: (&str, fn(&str, Box<dyn std::any::Any>) -> AnyAction) = (
                        stringify!($callback_name),
                        $callback_name,
                    );
                }

                let $arg = *args.downcast::<$arg_type>()
                    .expect(&format!(
                        "Invalid argument type: {}, expected: {}",
                        call_type,
                        stringify!($arg_type)));

                convert_impl($arg)
            }
        }

        Callback::new(stringify!($callback_name), convert_impl)
    }};
}

/// Creates a callback instance. Must accept a single argument, so `()`
/// should be used when no arguments are needed and tuples where
/// more than one value need to be passed.
///
/// # Example
///
/// ```ignore
/// callback!(task_done_callback(result: String) -> Action {
///     SomeAction { result }
/// })
///
/// callback!(multiple_arguments_callback((arg1: u64, arg2: u64)) -> Action {
///     MultipleArgumentsAction { value: arg1 + arg2 }
/// })
/// ```
#[macro_export]
macro_rules! callback {
    ($callback_name:ident(($($var:ident : $typ:ty),+)) -> $action_ty:ty $body:block) => {
        $crate::_callback!($callback_name, $action_ty, ($($var),+), ($($typ),+), $body)
    };
    ($callback_name:ident($var:ident : $typ:ty) -> $action_ty:ty $body:block) => {
        $crate::_callback!($callback_name, $action_ty, $var, $typ, $body)
    };
}
