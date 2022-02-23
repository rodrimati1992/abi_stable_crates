use super::TLFunctionQualifiers;

#[test]
fn test_qualifiers() {
    {
        let safe = TLFunctionQualifiers::NEW;
        let unsafe_ = TLFunctionQualifiers::NEW.set_unsafe();

        assert_eq!(safe, safe);
        assert_eq!(unsafe_, unsafe_);
        assert_ne!(safe, unsafe_);

        assert!(!safe.is_unsafe());
        assert!(unsafe_.is_unsafe());
    }
}
