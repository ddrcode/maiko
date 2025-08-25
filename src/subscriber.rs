use std::borrow::Cow;

pub(crate) struct Subscriber {
    name: Cow<'static, str>,
}
