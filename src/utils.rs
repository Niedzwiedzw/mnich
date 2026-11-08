#[allow(dead_code)]
/// Exactly like [std::option::Option::zip], but for [std::result::Result]
#[extension_traits::extension(pub trait ResultZipExt)]
impl<T, E> std::result::Result<T, E> {
    /// Exactly like [std::option::Option::zip], but for [std::result::Result]
    fn zip<O>(self, other: std::result::Result<O, E>) -> std::result::Result<(T, O), E> {
        self.and_then(|one| other.map(|other| (one, other)))
    }
}
