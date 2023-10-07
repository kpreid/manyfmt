use manyfmt::formats::Unquote;
use manyfmt::Refmt;

/// `.refmt()` can be called on a type that implements `Deref` to a formattable value,
/// and method lookup will accept this. This only works because the `F` parameter is a parameter
/// of the `Refmt` trait rather than the `refmt()` method of it.
#[test]
fn refmt_works_through_deref() {
    struct Container;
    impl std::ops::Deref for Container {
        type Target = str;

        fn deref(&self) -> &Self::Target {
            "hello"
        }
    }

    assert_eq!(format!("{:?}", Container.refmt(&Unquote)), "hello");
}
