use heapless::consts::U21;
use heapless::String;

pub type RowStorage = String<U21>;

#[cfg(test)]
mod tests {
    use super::*;
    use log::trace;

    #[test_case]
    fn row_storage() {
        trace!("row_storage");

        let rs = RowStorage::new();
        assert_eq!(rs.capacity(), 21);
        assert_eq!(rs.len(), 0);

        let rs = RowStorage::from("123-456-1234");
        assert_eq!(rs.capacity(), 21);
        assert_eq!(rs.len(), 12);
    }
}
